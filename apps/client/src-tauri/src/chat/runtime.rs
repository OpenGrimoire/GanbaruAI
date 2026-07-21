//! Per-thread provider session ownership, bounded command routing, and shutdown.

use crate::chat::events::{CanonicalEvent, CanonicalRuntimeEvent, NotificationEvent};
use crate::chat::models::{
    ChatCommandContext, ChatCommandId, ChatError, ChatErrorCode, ChatResult, ChatThreadId,
    ChatTurnId, DriverOperationReceipt, InterruptTurnRequest, ProviderSessionId,
    ProviderSessionSnapshot, ProviderSessionState, ResolveApprovalRequest, ResolveUserInputRequest,
    ResumeSessionRequest, RollbackRequest, SendTurnRequest, StartSessionRequest, SteerTurnRequest,
    StopSessionRequest, TurnDispatchReceipt,
};
use crate::chat::providers::{
    DriverCancellation, DriverFuture, DriverOperationContext, ProviderDriver, ProviderEventSink,
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use tauri::async_runtime::JoinHandle;
use tokio::sync::{mpsc, oneshot, Mutex as AsyncMutex, OwnedMutexGuard};

const DEFAULT_COMMAND_CAPACITY: usize = 32;
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const DEFAULT_STOP_TIMEOUT: Duration = Duration::from_secs(2);
const STOP_FLUSH_INTERVAL: Duration = Duration::from_millis(32);

pub enum ThreadRuntimeCommand {
    SessionState(ProviderSessionState),
    TurnActive(bool),
    PendingRequest(bool),
    TerminalLinkedOperation(bool),
    Touch,
    StartSession {
        driver: Box<dyn ProviderDriver>,
        request: StartSessionRequest,
        event_sink: Arc<dyn ProviderEventSink>,
        context: DriverOperationContext,
        response: oneshot::Sender<ChatResult<ProviderSessionSnapshot>>,
    },
    ResumeSession {
        driver: Box<dyn ProviderDriver>,
        request: ResumeSessionRequest,
        event_sink: Arc<dyn ProviderEventSink>,
        context: DriverOperationContext,
        response: oneshot::Sender<ChatResult<ProviderSessionSnapshot>>,
    },
    SendTurn {
        request: SendTurnRequest,
        context: DriverOperationContext,
        response: oneshot::Sender<ChatResult<TurnDispatchReceipt>>,
    },
    SteerTurn {
        request: SteerTurnRequest,
        context: DriverOperationContext,
        response: oneshot::Sender<ChatResult<DriverOperationReceipt>>,
    },
    ResolveApproval {
        request: ResolveApprovalRequest,
        context: DriverOperationContext,
        response: oneshot::Sender<ChatResult<DriverOperationReceipt>>,
    },
    ResolveUserInput {
        request: ResolveUserInputRequest,
        context: DriverOperationContext,
        response: oneshot::Sender<ChatResult<DriverOperationReceipt>>,
    },
    InterruptTurn {
        request: InterruptTurnRequest,
        context: DriverOperationContext,
        response: oneshot::Sender<ChatResult<DriverOperationReceipt>>,
    },
    Rollback {
        request: RollbackRequest,
        context: DriverOperationContext,
        response: oneshot::Sender<ChatResult<DriverOperationReceipt>>,
    },
    StopSession {
        force: bool,
        context: DriverOperationContext,
        response: oneshot::Sender<ChatResult<DriverOperationReceipt>>,
    },
    Shutdown {
        deadline: Instant,
        response: oneshot::Sender<ChatResult<()>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadRuntimeSnapshot {
    pub session_id: Option<ProviderSessionId>,
    pub session_state: ProviderSessionState,
    pub capabilities: crate::chat::models::ProviderCapabilities,
    pub active_turn_id: Option<ChatTurnId>,
    pub turn_active: bool,
    pub pending_request: bool,
    pub terminal_linked_operation: bool,
    pub accepting_commands: bool,
    pub generation: u64,
    pub last_activity: Instant,
}

impl ThreadRuntimeSnapshot {
    pub fn can_idle_stop(&self, now: Instant, idle_timeout: Duration) -> bool {
        self.accepting_commands
            && self.session_state == ProviderSessionState::Ready
            && !self.turn_active
            && !self.pending_request
            && !self.terminal_linked_operation
            && now.saturating_duration_since(self.last_activity) >= idle_timeout
    }
}

pub struct ThreadRuntimeOwner {
    thread_id: ChatThreadId,
    command_sender: mpsc::Sender<ThreadRuntimeCommand>,
    operation_lock: Arc<AsyncMutex<()>>,
    snapshot: Arc<Mutex<ThreadRuntimeSnapshot>>,
    worker_task: Mutex<Option<JoinHandle<()>>>,
}

impl ThreadRuntimeOwner {
    pub fn thread_id(&self) -> &ChatThreadId {
        &self.thread_id
    }

    pub fn snapshot(&self) -> ChatResult<ThreadRuntimeSnapshot> {
        self.snapshot
            .lock()
            .map(|snapshot| snapshot.clone())
            .map_err(|_| runtime_state_error())
    }

    pub fn try_command(&self, command: ThreadRuntimeCommand) -> ChatResult<()> {
        if !self.snapshot()?.accepting_commands {
            return Err(runtime_unavailable());
        }
        self.try_send(command)
    }

    pub async fn lock_operation(&self) -> OwnedMutexGuard<()> {
        Arc::clone(&self.operation_lock).lock_owned().await
    }

    pub async fn start_session(
        &self,
        driver: Box<dyn ProviderDriver>,
        request: StartSessionRequest,
        event_sink: Arc<dyn ProviderEventSink>,
        context: DriverOperationContext,
    ) -> ChatResult<ProviderSessionSnapshot> {
        let _guard = self.lock_operation().await;
        let (response, receiver) = oneshot::channel();
        self.try_command(ThreadRuntimeCommand::StartSession {
            driver,
            request,
            event_sink,
            context,
            response,
        })?;
        receive_response(receiver).await
    }

    pub async fn send_turn(
        &self,
        request: SendTurnRequest,
        context: DriverOperationContext,
    ) -> ChatResult<TurnDispatchReceipt> {
        let _guard = self.lock_operation().await;
        let (response, receiver) = oneshot::channel();
        self.try_command(ThreadRuntimeCommand::SendTurn {
            request,
            context,
            response,
        })?;
        receive_response(receiver).await
    }

    pub async fn resume_session(
        &self,
        driver: Box<dyn ProviderDriver>,
        request: ResumeSessionRequest,
        event_sink: Arc<dyn ProviderEventSink>,
        context: DriverOperationContext,
    ) -> ChatResult<ProviderSessionSnapshot> {
        let _guard = self.lock_operation().await;
        let (response, receiver) = oneshot::channel();
        self.try_command(ThreadRuntimeCommand::ResumeSession {
            driver,
            request,
            event_sink,
            context,
            response,
        })?;
        receive_response(receiver).await
    }

    pub async fn steer_turn(
        &self,
        request: SteerTurnRequest,
        context: DriverOperationContext,
    ) -> ChatResult<DriverOperationReceipt> {
        let _guard = self.lock_operation().await;
        let (response, receiver) = oneshot::channel();
        self.try_command(ThreadRuntimeCommand::SteerTurn {
            request,
            context,
            response,
        })?;
        receive_response(receiver).await
    }

    pub async fn resolve_approval(
        &self,
        request: ResolveApprovalRequest,
        context: DriverOperationContext,
    ) -> ChatResult<DriverOperationReceipt> {
        let _guard = self.lock_operation().await;
        let (response, receiver) = oneshot::channel();
        self.try_command(ThreadRuntimeCommand::ResolveApproval {
            request,
            context,
            response,
        })?;
        receive_response(receiver).await
    }

    pub async fn interrupt_turn(
        &self,
        request: InterruptTurnRequest,
        context: DriverOperationContext,
    ) -> ChatResult<DriverOperationReceipt> {
        let _guard = self.lock_operation().await;
        let (response, receiver) = oneshot::channel();
        self.try_command(ThreadRuntimeCommand::InterruptTurn {
            request,
            context,
            response,
        })?;
        receive_response(receiver).await
    }

    pub async fn resolve_user_input(
        &self,
        request: ResolveUserInputRequest,
        context: DriverOperationContext,
    ) -> ChatResult<DriverOperationReceipt> {
        let _guard = self.lock_operation().await;
        let (response, receiver) = oneshot::channel();
        self.try_command(ThreadRuntimeCommand::ResolveUserInput {
            request,
            context,
            response,
        })?;
        receive_response(receiver).await
    }

    pub async fn rollback(
        &self,
        request: RollbackRequest,
        context: DriverOperationContext,
    ) -> ChatResult<DriverOperationReceipt> {
        let _guard = self.lock_operation().await;
        let (response, receiver) = oneshot::channel();
        self.try_command(ThreadRuntimeCommand::Rollback {
            request,
            context,
            response,
        })?;
        receive_response(receiver).await
    }

    pub async fn stop_session(
        &self,
        force: bool,
        context: DriverOperationContext,
    ) -> ChatResult<DriverOperationReceipt> {
        let _guard = self.lock_operation().await;
        let (response, receiver) = oneshot::channel();
        self.try_command(ThreadRuntimeCommand::StopSession {
            force,
            context,
            response,
        })?;
        receive_response(receiver).await
    }

    fn try_send(&self, command: ThreadRuntimeCommand) -> ChatResult<()> {
        self.command_sender
            .try_send(command)
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => ChatError::new(
                    ChatErrorCode::Busy,
                    "Chat thread command queue is full",
                    true,
                ),
                mpsc::error::TrySendError::Closed(_) => runtime_unavailable(),
            })
    }

    fn stop_accepting(&self) -> ChatResult<()> {
        self.snapshot
            .lock()
            .map_err(|_| runtime_state_error())?
            .accepting_commands = false;
        Ok(())
    }

    fn abort_worker(&self) -> ChatResult<()> {
        if let Some(task) = self
            .worker_task
            .lock()
            .map_err(|_| runtime_state_error())?
            .take()
        {
            task.abort();
        }
        Ok(())
    }
}

pub struct ChatRuntimeRegistry {
    owners: Mutex<HashMap<ChatThreadId, Arc<ThreadRuntimeOwner>>>,
    idle_timeout: Duration,
}

impl Default for ChatRuntimeRegistry {
    fn default() -> Self {
        Self {
            owners: Mutex::new(HashMap::new()),
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
        }
    }
}

impl ChatRuntimeRegistry {
    pub fn owner(&self, thread_id: ChatThreadId) -> ChatResult<Arc<ThreadRuntimeOwner>> {
        self.owner_with_capacity(thread_id, DEFAULT_COMMAND_CAPACITY)
    }

    fn owner_with_capacity(
        &self,
        thread_id: ChatThreadId,
        capacity: usize,
    ) -> ChatResult<Arc<ThreadRuntimeOwner>> {
        self.owner_with_start_gate(thread_id, capacity, None)
    }

    fn owner_with_start_gate(
        &self,
        thread_id: ChatThreadId,
        capacity: usize,
        start_gate: Option<oneshot::Receiver<()>>,
    ) -> ChatResult<Arc<ThreadRuntimeOwner>> {
        let mut owners = self.owners.lock().map_err(|_| runtime_state_error())?;
        if let Some(owner) = owners.get(&thread_id) {
            return Ok(Arc::clone(owner));
        }
        let (sender, receiver) = mpsc::channel(capacity.max(1));
        let snapshot = Arc::new(Mutex::new(ThreadRuntimeSnapshot {
            session_id: None,
            session_state: ProviderSessionState::Stopped,
            capabilities: crate::chat::models::ProviderCapabilities::default(),
            active_turn_id: None,
            turn_active: false,
            pending_request: false,
            terminal_linked_operation: false,
            accepting_commands: true,
            generation: 0,
            last_activity: Instant::now(),
        }));
        let generation = Arc::new(AtomicU64::new(0));
        let worker_thread_id = thread_id.clone();
        let worker_snapshot = Arc::clone(&snapshot);
        let idle_timeout = self.idle_timeout;
        let worker = tauri::async_runtime::spawn(async move {
            if let Some(start_gate) = start_gate {
                let _ = start_gate.await;
            }
            run_thread_runtime(
                worker_thread_id,
                receiver,
                worker_snapshot,
                generation,
                idle_timeout,
            )
            .await;
        });
        let owner = Arc::new(ThreadRuntimeOwner {
            thread_id: thread_id.clone(),
            command_sender: sender,
            operation_lock: Arc::new(AsyncMutex::new(())),
            snapshot,
            worker_task: Mutex::new(Some(worker)),
        });
        owners.insert(thread_id, Arc::clone(&owner));
        Ok(owner)
    }

    pub fn owners(&self) -> ChatResult<Vec<Arc<ThreadRuntimeOwner>>> {
        self.owners
            .lock()
            .map(|owners| owners.values().cloned().collect())
            .map_err(|_| runtime_state_error())
    }

    pub async fn shutdown_and_wait(&self, timeout: Duration) -> ChatResult<()> {
        let owners = self.owners()?;
        for owner in &owners {
            owner.stop_accepting()?;
        }
        let shutdown = async {
            let deadline = Instant::now() + timeout;
            let mut receivers = Vec::with_capacity(owners.len());
            for owner in &owners {
                let (response, receiver) = oneshot::channel();
                owner
                    .command_sender
                    .send(ThreadRuntimeCommand::Shutdown { deadline, response })
                    .await
                    .map_err(|_| runtime_unavailable())?;
                receivers.push(receiver);
            }
            for receiver in receivers {
                receive_response(receiver).await?;
            }
            Ok(())
        };
        match tokio::time::timeout(timeout, shutdown).await {
            Ok(result) => result,
            Err(_) => {
                for owner in owners {
                    owner.abort_worker()?;
                }
                Err(runtime_timeout())
            }
        }
    }
}

struct RuntimeWorker {
    thread_id: ChatThreadId,
    driver: Option<Box<dyn ProviderDriver>>,
    event_sink: Option<Arc<dyn ProviderEventSink>>,
    session: Option<ProviderSessionSnapshot>,
    snapshot: Arc<Mutex<ThreadRuntimeSnapshot>>,
    generation: Arc<AtomicU64>,
}

impl RuntimeWorker {
    fn new(
        thread_id: ChatThreadId,
        snapshot: Arc<Mutex<ThreadRuntimeSnapshot>>,
        generation: Arc<AtomicU64>,
    ) -> Self {
        Self {
            thread_id,
            driver: None,
            event_sink: None,
            session: None,
            snapshot,
            generation,
        }
    }

    async fn handle(&mut self, command: ThreadRuntimeCommand) -> bool {
        self.touch();
        match command {
            ThreadRuntimeCommand::SessionState(value) => self.update(|state| {
                state.session_state = value;
            }),
            ThreadRuntimeCommand::TurnActive(value) => self.update(|state| {
                state.turn_active = value;
                if !value {
                    state.active_turn_id = None;
                }
            }),
            ThreadRuntimeCommand::PendingRequest(value) => self.update(|state| {
                state.pending_request = value;
            }),
            ThreadRuntimeCommand::TerminalLinkedOperation(value) => self.update(|state| {
                state.terminal_linked_operation = value;
            }),
            ThreadRuntimeCommand::Touch => {}
            ThreadRuntimeCommand::StartSession {
                mut driver,
                request,
                event_sink,
                context,
                response,
            } => {
                if self.driver.is_some()
                    && !matches!(
                        self.session_state(),
                        ProviderSessionState::Stopped | ProviderSessionState::Failed
                    )
                {
                    let _ = response.send(Err(runtime_invalid_state(
                        "Chat provider session is already running",
                    )));
                    return false;
                }
                let generation = self.next_generation();
                self.driver.take();
                self.session = None;
                self.update(|state| {
                    state.session_id = None;
                    state.session_state = ProviderSessionState::Starting;
                    state.capabilities = crate::chat::models::ProviderCapabilities::default();
                    state.turn_active = false;
                    state.active_turn_id = None;
                    state.pending_request = false;
                    state.generation = generation;
                });
                let sink: Arc<dyn ProviderEventSink> = Arc::new(GenerationEventSink {
                    thread_id: self.thread_id.clone(),
                    generation,
                    current_generation: Arc::clone(&self.generation),
                    snapshot: Arc::clone(&self.snapshot),
                    inner: event_sink,
                });
                self.event_sink = Some(Arc::clone(&sink));
                let result =
                    run_driver_operation(&context, driver.start_session(request, sink, &context))
                        .await;
                if result.is_err() {
                    let _ = self.flush_events(&context).await;
                }
                match &result {
                    Ok(session) => {
                        self.session = Some(session.clone());
                        self.update(|state| {
                            state.session_id = Some(session.session_id.clone());
                            state.session_state = session.state;
                            state.capabilities = session.capabilities.clone();
                        });
                    }
                    Err(_) => {
                        self.update(|state| state.session_state = ProviderSessionState::Failed)
                    }
                }
                self.driver = Some(driver);
                let _ = response.send(result);
            }
            ThreadRuntimeCommand::ResumeSession {
                mut driver,
                request,
                event_sink,
                context,
                response,
            } => {
                if self.driver.is_some()
                    && !matches!(
                        self.session_state(),
                        ProviderSessionState::Stopped | ProviderSessionState::Failed
                    )
                {
                    let _ = response.send(Err(runtime_invalid_state(
                        "Chat provider session is already running",
                    )));
                    return false;
                }
                let generation = self.next_generation();
                self.driver.take();
                self.session = None;
                self.update(|state| {
                    state.session_id = None;
                    state.session_state = ProviderSessionState::Starting;
                    state.capabilities = crate::chat::models::ProviderCapabilities::default();
                    state.turn_active = false;
                    state.active_turn_id = None;
                    state.pending_request = false;
                    state.generation = generation;
                });
                let sink: Arc<dyn ProviderEventSink> = Arc::new(GenerationEventSink {
                    thread_id: self.thread_id.clone(),
                    generation,
                    current_generation: Arc::clone(&self.generation),
                    snapshot: Arc::clone(&self.snapshot),
                    inner: event_sink,
                });
                self.event_sink = Some(Arc::clone(&sink));
                let result =
                    run_driver_operation(&context, driver.resume_session(request, sink, &context))
                        .await;
                if result.is_err() {
                    let _ = self.flush_events(&context).await;
                }
                match &result {
                    Ok(session) => {
                        self.session = Some(session.clone());
                        self.update(|state| {
                            state.session_id = Some(session.session_id.clone());
                            state.session_state = session.state;
                            state.capabilities = session.capabilities.clone();
                        });
                    }
                    Err(_) => {
                        self.update(|state| state.session_state = ProviderSessionState::Failed)
                    }
                }
                self.driver = Some(driver);
                let _ = response.send(result);
            }
            ThreadRuntimeCommand::SendTurn {
                request,
                context,
                response,
            } => {
                if self.session_state() != ProviderSessionState::Ready {
                    let _ = response.send(Err(runtime_invalid_state(
                        "Chat provider session is not ready for a new turn",
                    )));
                    return false;
                }
                let turn_id = request.turn_id.clone();
                self.update(|state| {
                    state.session_state = ProviderSessionState::Active;
                    state.active_turn_id = Some(turn_id);
                    state.turn_active = true;
                });
                let result = match self.driver.as_mut() {
                    Some(driver) => {
                        run_driver_operation(&context, driver.send_turn(request, &context)).await
                    }
                    None => Err(runtime_unavailable()),
                };
                if result.is_err() {
                    let _ = self.flush_events(&context).await;
                    self.update(|state| state.session_state = ProviderSessionState::Failed);
                }
                let _ = response.send(result);
            }
            ThreadRuntimeCommand::SteerTurn {
                request,
                context,
                response,
            } => {
                if self.session_state() != ProviderSessionState::Active {
                    let _ = response.send(Err(runtime_invalid_state(
                        "Chat provider session has no steerable turn",
                    )));
                    return false;
                }
                let result = match self.driver.as_mut() {
                    Some(driver) => {
                        run_driver_operation(&context, driver.steer_turn(request, &context)).await
                    }
                    None => Err(runtime_unavailable()),
                };
                let _ = response.send(result);
            }
            ThreadRuntimeCommand::ResolveApproval {
                request,
                context,
                response,
            } => {
                if self.session_state() != ProviderSessionState::WaitingForApproval {
                    let _ = response.send(Err(runtime_invalid_state(
                        "Chat provider session has no pending approval",
                    )));
                    return false;
                }
                let result = match self.driver.as_mut() {
                    Some(driver) => {
                        run_driver_operation(&context, driver.resolve_approval(request, &context))
                            .await
                    }
                    None => Err(runtime_unavailable()),
                };
                if result.is_ok() {
                    self.update(|state| {
                        state.pending_request = false;
                        if matches!(
                            state.session_state,
                            ProviderSessionState::WaitingForApproval
                                | ProviderSessionState::WaitingForUserInput
                        ) {
                            state.session_state = ProviderSessionState::Active;
                        }
                    });
                }
                let _ = response.send(result);
            }
            ThreadRuntimeCommand::ResolveUserInput {
                request,
                context,
                response,
            } => {
                if self.session_state() != ProviderSessionState::WaitingForUserInput {
                    let _ = response.send(Err(runtime_invalid_state(
                        "Chat provider session has no pending user input request",
                    )));
                    return false;
                }
                let result = match self.driver.as_mut() {
                    Some(driver) => {
                        run_driver_operation(&context, driver.resolve_user_input(request, &context))
                            .await
                    }
                    None => Err(runtime_unavailable()),
                };
                if result.is_ok() {
                    self.update(|state| {
                        state.pending_request = false;
                        state.session_state = ProviderSessionState::Active;
                    });
                }
                let _ = response.send(result);
            }
            ThreadRuntimeCommand::InterruptTurn {
                request,
                context,
                response,
            } => {
                if !matches!(
                    self.session_state(),
                    ProviderSessionState::Active
                        | ProviderSessionState::WaitingForApproval
                        | ProviderSessionState::WaitingForUserInput
                ) {
                    let _ = response.send(Err(runtime_invalid_state(
                        "Chat provider session has no interruptible turn",
                    )));
                    return false;
                }
                let previous_state = self.session_state();
                self.update(|state| state.session_state = ProviderSessionState::Stopping);
                let result = match self.driver.as_mut() {
                    Some(driver) => {
                        run_driver_operation(&context, driver.interrupt_turn(request, &context))
                            .await
                    }
                    None => Err(runtime_unavailable()),
                };
                if result.is_err() {
                    self.update(|state| state.session_state = previous_state);
                }
                let _ = response.send(result);
            }
            ThreadRuntimeCommand::Rollback {
                request,
                context,
                response,
            } => {
                if self.session_state() != ProviderSessionState::Ready {
                    let _ = response.send(Err(runtime_invalid_state(
                        "Chat provider session must be ready before rollback",
                    )));
                    return false;
                }
                let result = match self.driver.as_mut() {
                    Some(driver) => {
                        run_driver_operation(&context, driver.rollback(request, &context)).await
                    }
                    None => Err(runtime_unavailable()),
                };
                let _ = response.send(result);
            }
            ThreadRuntimeCommand::StopSession {
                force,
                context,
                response,
            } => {
                let result = self.stop_driver(force, context).await;
                let receipt = result.map(|()| DriverOperationReceipt {
                    accepted: true,
                    operation_id: "runtime-stop".to_string(),
                    detail: None,
                });
                let _ = response.send(receipt);
            }
            ThreadRuntimeCommand::Shutdown { deadline, response } => {
                self.update(|state| state.accepting_commands = false);
                let context = operation_context("runtime-shutdown", deadline);
                let result = self.stop_driver(true, context).await;
                let _ = response.send(result);
                return true;
            }
        }
        false
    }

    async fn stop_driver(
        &mut self,
        force: bool,
        context: DriverOperationContext,
    ) -> ChatResult<()> {
        let initial_flush = self.flush_events(&context).await;
        let Some(mut driver) = self.driver.take() else {
            self.session = None;
            self.event_sink = None;
            self.update(|state| {
                state.session_id = None;
                state.session_state = ProviderSessionState::Stopped;
                state.capabilities = crate::chat::models::ProviderCapabilities::default();
                state.active_turn_id = None;
                state.turn_active = false;
                state.pending_request = false;
            });
            return initial_flush;
        };
        self.update(|state| state.session_state = ProviderSessionState::Stopping);
        if let (Some(session), Some(turn_id)) = (self.session.as_ref(), self.active_turn_id()) {
            let request = InterruptTurnRequest {
                command: ChatCommandContext {
                    client_command_id: ChatCommandId::new("runtime-stop-interrupt")
                        .map_err(runtime_identifier_error)?,
                    expected_thread_revision: None,
                },
                session_id: session.session_id.clone(),
                turn_id,
            };
            let _ = run_driver_operation_with_flush(
                &context,
                driver.interrupt_turn(request, &context),
                self.event_sink.clone(),
            )
            .await;
        }
        let stop_result = if let Some(session) = self.session.as_ref() {
            let request = StopSessionRequest {
                session_id: session.session_id.clone(),
                force,
            };
            run_driver_operation_with_flush(
                &context,
                driver.stop_session(request, &context),
                self.event_sink.clone(),
            )
            .await
            .map(|_| ())
        } else {
            Ok(())
        };
        let final_flush = self.flush_events(&context).await;
        let result = initial_flush.and(stop_result).and(final_flush);
        self.next_generation();
        self.session = None;
        self.event_sink = None;
        self.update(|state| {
            state.session_id = None;
            state.session_state = if result.is_ok() {
                ProviderSessionState::Stopped
            } else {
                ProviderSessionState::Failed
            };
            state.capabilities = crate::chat::models::ProviderCapabilities::default();
            state.active_turn_id = None;
            state.turn_active = false;
            state.pending_request = false;
        });
        result
    }

    async fn idle_stop(&mut self) {
        let deadline = Instant::now() + DEFAULT_STOP_TIMEOUT;
        let _ = self
            .stop_driver(false, operation_context("runtime-idle-stop", deadline))
            .await;
    }

    fn can_idle_stop(&self, idle_timeout: Duration) -> bool {
        self.driver.is_some()
            && self
                .snapshot
                .lock()
                .map(|snapshot| snapshot.can_idle_stop(Instant::now(), idle_timeout))
                .unwrap_or(false)
    }

    fn idle_deadline(&self, idle_timeout: Duration) -> Option<Instant> {
        self.driver.as_ref()?;
        self.snapshot
            .lock()
            .ok()
            .map(|snapshot| snapshot.last_activity + idle_timeout)
    }

    fn active_turn_id(&self) -> Option<ChatTurnId> {
        self.snapshot
            .lock()
            .ok()
            .and_then(|snapshot| snapshot.active_turn_id.clone())
    }

    async fn flush_events(&mut self, context: &DriverOperationContext) -> ChatResult<()> {
        match self.event_sink.as_ref() {
            Some(sink) => run_driver_operation(context, sink.flush()).await,
            None => Ok(()),
        }
    }

    fn session_state(&self) -> ProviderSessionState {
        self.snapshot
            .lock()
            .map(|snapshot| snapshot.session_state)
            .unwrap_or(ProviderSessionState::Failed)
    }

    fn next_generation(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::AcqRel) + 1
    }

    fn touch(&self) {
        self.update(|state| state.last_activity = Instant::now());
    }

    fn update(&self, update: impl FnOnce(&mut ThreadRuntimeSnapshot)) {
        if let Ok(mut snapshot) = self.snapshot.lock() {
            update(&mut snapshot);
        }
    }
}

async fn run_thread_runtime(
    thread_id: ChatThreadId,
    mut receiver: mpsc::Receiver<ThreadRuntimeCommand>,
    snapshot: Arc<Mutex<ThreadRuntimeSnapshot>>,
    generation: Arc<AtomicU64>,
    idle_timeout: Duration,
) {
    let mut worker = RuntimeWorker::new(thread_id, snapshot, generation);
    loop {
        let command = match worker.idle_deadline(idle_timeout) {
            Some(deadline) => match tokio::time::timeout_at(
                tokio::time::Instant::from_std(deadline),
                receiver.recv(),
            )
            .await
            {
                Ok(command) => command,
                Err(_) => {
                    if worker.can_idle_stop(idle_timeout) {
                        worker.idle_stop().await;
                    }
                    continue;
                }
            },
            None => receiver.recv().await,
        };
        let Some(command) = command else {
            let deadline = Instant::now() + DEFAULT_STOP_TIMEOUT;
            let _ = worker
                .stop_driver(true, operation_context("runtime-channel-close", deadline))
                .await;
            worker.update(|state| state.accepting_commands = false);
            return;
        };
        if worker.handle(command).await {
            return;
        }
    }
}

struct GenerationEventSink {
    thread_id: ChatThreadId,
    generation: u64,
    current_generation: Arc<AtomicU64>,
    snapshot: Arc<Mutex<ThreadRuntimeSnapshot>>,
    inner: Arc<dyn ProviderEventSink>,
}

impl ProviderEventSink for GenerationEventSink {
    fn emit<'a>(&'a self, event: CanonicalRuntimeEvent) -> DriverFuture<'a, ()> {
        Box::pin(async move {
            if self.current_generation.load(Ordering::Acquire) != self.generation {
                self.inner
                    .emit(late_event_warning(
                        event,
                        &self.thread_id,
                        "inactive_session_generation",
                    ))
                    .await?;
                return Err(stale_event_error());
            }
            if event.thread_id != self.thread_id {
                self.inner
                    .emit(late_event_warning(
                        event,
                        &self.thread_id,
                        "mismatched_thread",
                    ))
                    .await?;
                return Err(stale_event_error());
            }
            if !event_matches_active_turn(&self.snapshot, &event) {
                self.inner
                    .emit(late_event_warning(
                        event,
                        &self.thread_id,
                        "settled_or_mismatched_turn",
                    ))
                    .await?;
                return Err(stale_event_error());
            }
            self.inner.emit(event.clone()).await?;
            if self.current_generation.load(Ordering::Acquire) == self.generation {
                update_snapshot_for_event(&self.snapshot, &event);
            }
            Ok(())
        })
    }

    fn flush(&self) -> DriverFuture<'_, ()> {
        self.inner.flush()
    }
}

fn event_matches_active_turn(
    snapshot: &Arc<Mutex<ThreadRuntimeSnapshot>>,
    event: &CanonicalRuntimeEvent,
) -> bool {
    let requires_active_turn = matches!(
        event.event,
        CanonicalEvent::TurnStarted(_)
            | CanonicalEvent::TurnCompleted(_)
            | CanonicalEvent::TurnAborted(_)
            | CanonicalEvent::PlanUpdated(_)
            | CanonicalEvent::ProposedPlanDelta(_)
            | CanonicalEvent::ProposedPlanCompleted(_)
            | CanonicalEvent::DiffUpdated(_)
            | CanonicalEvent::ItemStarted(_)
            | CanonicalEvent::ItemUpdated(_)
            | CanonicalEvent::ItemCompleted(_)
            | CanonicalEvent::ContentDelta(_)
            | CanonicalEvent::RequestOpened(_)
            | CanonicalEvent::RequestResolved(_)
            | CanonicalEvent::UserInputRequested(_)
            | CanonicalEvent::UserInputResolved(_)
            | CanonicalEvent::TaskLifecycle(_)
            | CanonicalEvent::HookLifecycle(_)
            | CanonicalEvent::ToolProgress(_)
            | CanonicalEvent::FilesPersisted(_)
    );
    if !requires_active_turn {
        return true;
    }
    snapshot
        .lock()
        .map(|state| event.turn_id.as_ref() == state.active_turn_id.as_ref())
        .unwrap_or(false)
}

fn late_event_warning(
    mut event: CanonicalRuntimeEvent,
    thread_id: &ChatThreadId,
    reason: &'static str,
) -> CanonicalRuntimeEvent {
    event.thread_id = thread_id.clone();
    event.turn_id = None;
    event.provider_turn_id = None;
    event.provider_item_id = None;
    event.provider_request_id = None;
    event.provider_task_id = None;
    event.provider_reference = None;
    event.redacted_diagnostic = None;
    event.event = CanonicalEvent::RuntimeWarning(NotificationEvent {
        code: "late_provider_event".to_string(),
        title: "Late provider event ignored".to_string(),
        detail: Some(reason.to_string()),
    });
    event
}

fn update_snapshot_for_event(
    snapshot: &Arc<Mutex<ThreadRuntimeSnapshot>>,
    event: &CanonicalRuntimeEvent,
) {
    let Ok(mut state) = snapshot.lock() else {
        return;
    };
    state.last_activity = Instant::now();
    match &event.event {
        CanonicalEvent::SessionStarted(event) => {
            state.session_id = Some(event.session_id.clone());
            state.session_state = event.state;
            state.capabilities = event.capability_overrides.clone();
        }
        CanonicalEvent::SessionStateChanged(event) => state.session_state = event.state,
        CanonicalEvent::SessionExited(event) => {
            state.session_id = None;
            state.session_state = if event.expected {
                ProviderSessionState::Stopped
            } else {
                ProviderSessionState::Failed
            };
            state.active_turn_id = None;
            state.turn_active = false;
            state.pending_request = false;
            state.capabilities = crate::chat::models::ProviderCapabilities::default();
        }
        CanonicalEvent::TurnStarted(_) => {
            state.active_turn_id = event.turn_id.clone();
            state.turn_active = true;
            state.session_state = ProviderSessionState::Active;
        }
        CanonicalEvent::TurnCompleted(_) | CanonicalEvent::TurnAborted(_) => {
            state.active_turn_id = None;
            state.turn_active = false;
            state.pending_request = false;
            state.session_state = ProviderSessionState::Ready;
        }
        CanonicalEvent::RequestOpened(_) => {
            state.pending_request = true;
            state.session_state = ProviderSessionState::WaitingForApproval;
        }
        CanonicalEvent::UserInputRequested(_) => {
            state.pending_request = true;
            state.session_state = ProviderSessionState::WaitingForUserInput;
        }
        CanonicalEvent::RequestResolved(_) | CanonicalEvent::UserInputResolved(_) => {
            state.pending_request = false;
            state.session_state = ProviderSessionState::Active;
        }
        CanonicalEvent::RuntimeError(event) if !event.recoverable => {
            state.session_state = ProviderSessionState::Failed;
        }
        _ => {}
    }
}

async fn run_driver_operation<T>(
    context: &DriverOperationContext,
    future: DriverFuture<'_, T>,
) -> ChatResult<T> {
    tokio::time::timeout_at(tokio::time::Instant::from_std(context.deadline), future)
        .await
        .map_err(|_| runtime_timeout())?
}

async fn run_driver_operation_with_flush<T>(
    context: &DriverOperationContext,
    mut future: DriverFuture<'_, T>,
    event_sink: Option<Arc<dyn ProviderEventSink>>,
) -> ChatResult<T> {
    loop {
        let now = Instant::now();
        if now >= context.deadline {
            return Err(runtime_timeout());
        }
        let tick_deadline = (now + STOP_FLUSH_INTERVAL).min(context.deadline);
        match tokio::time::timeout_at(tokio::time::Instant::from_std(tick_deadline), &mut future)
            .await
        {
            Ok(result) => return result,
            Err(_) if tick_deadline == context.deadline => return Err(runtime_timeout()),
            Err(_) => {
                if let Some(sink) = event_sink.as_ref() {
                    run_driver_operation(context, sink.flush()).await?;
                }
            }
        }
    }
}

fn operation_context(operation_id: &str, deadline: Instant) -> DriverOperationContext {
    DriverOperationContext {
        operation_id: operation_id.to_string(),
        deadline,
        cancellation: DriverCancellation::default(),
    }
}

async fn receive_response<T>(receiver: oneshot::Receiver<ChatResult<T>>) -> ChatResult<T> {
    receiver.await.map_err(|_| runtime_unavailable())?
}

fn runtime_state_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Chat runtime registry is unavailable",
        false,
    )
}

fn runtime_unavailable() -> ChatError {
    ChatError::new(
        ChatErrorCode::DriverUnavailable,
        "Chat thread session is not accepting commands",
        true,
    )
}

fn runtime_timeout() -> ChatError {
    ChatError::new(
        ChatErrorCode::DriverUnavailable,
        "Chat provider operation exceeded its deadline",
        true,
    )
}

fn runtime_invalid_state(message: &'static str) -> ChatError {
    ChatError::new(ChatErrorCode::InvalidStateTransition, message, true)
}

fn runtime_identifier_error<T>(_error: T) -> ChatError {
    runtime_state_error()
}

fn stale_event_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Conflict,
        "Chat provider event belongs to an inactive session generation",
        false,
    )
}

#[cfg(test)]
impl ChatRuntimeRegistry {
    pub fn owner_for_test(
        &self,
        thread_id: ChatThreadId,
        capacity: usize,
    ) -> ChatResult<(Arc<ThreadRuntimeOwner>, oneshot::Sender<()>)> {
        let (start, wait) = oneshot::channel();
        self.owner_with_start_gate(thread_id, capacity, Some(wait))
            .map(|owner| (owner, start))
    }

    pub fn with_idle_timeout_for_test(idle_timeout: Duration) -> Self {
        Self {
            owners: Mutex::new(HashMap::new()),
            idle_timeout,
        }
    }
}
