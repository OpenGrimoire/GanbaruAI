<div class="activity-indicator" aria-hidden="true">
  <span></span>
  <span></span>
  <span></span>
</div>

<style>
  .activity-indicator {
    display: flex;
    height: 1rem;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
  }

  .activity-indicator span {
    width: 0.4rem;
    height: 0.4rem;
    border-radius: 50%;
    background: var(--primary);
    animation: activity-pulse 1.2s ease-in-out infinite;
  }

  .activity-indicator span:nth-child(2) { animation-delay: 0.15s; }
  .activity-indicator span:nth-child(3) { animation-delay: 0.3s; }

  @keyframes activity-pulse {
    0%, 60%, 100% { opacity: 0.3; transform: translateY(0); }
    30% { opacity: 1; transform: translateY(-0.2rem); }
  }

  @media (prefers-reduced-motion: reduce) {
    .activity-indicator span {
      animation: none;
      opacity: 0.65;
    }
  }
</style>
