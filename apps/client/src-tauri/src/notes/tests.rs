use super::models::{
    NoteAppendBlockChildren, NoteBlockUpdate, NoteBlockWrite, NoteChildPageFromBlockCreate,
    NoteCommentAnchorCreate, NoteCommentCreate, NoteCommentThreadReadUpdate, NoteCommentUpdate,
    NoteDataSourceBoardConfigurationUpdate, NoteDataSourceBoardRowMove,
    NoteDataSourceBoardViewUpdate, NoteDataSourceButtonClick,
    NoteDataSourceCalendarConfigurationUpdate, NoteDataSourceCalendarViewUpdate,
    NoteDataSourceGalleryConfigurationUpdate, NoteDataSourceGalleryViewUpdate,
    NoteDataSourceListConfigurationUpdate, NoteDataSourceListViewUpdate,
    NoteDataSourceRowPageCreate, NoteDataSourceRowPropertyUpdate, NoteDataSourceSchemaUpdate,
    NoteDataSourceTableConfigurationUpdate, NoteDataSourceTableFilter, NoteDataSourceTableSort,
    NoteDataSourceTableViewUpdate, NoteDataSourceTemplateApply,
    NoteDataSourceTemplateCreateFromRow, NoteDataSourceTimelineConfigurationUpdate,
    NoteDataSourceTimelineViewUpdate, NoteDatabaseCreate, NoteDuplicateBlock, NoteDuplicateBlocks,
    NoteDuplicatePage, NoteDuplicatedBlockId, NoteLinkedDatabaseCreate, NoteLocalUserUpdate,
    NoteMentionNotificationDeliveryUpdate, NoteMoveBlock, NoteMoveBlocks, NoteMovePage,
    NotePageCreate, NotePageHistoryCopyBlocks, NotePageHistorySettingsUpdate,
    NotePageTemplateApply, NotePageTemplateCreateFromPage, NotePageTemplateDuplicate,
    NotePageTemplateUpdate, NoteParent, NoteSidebarPagesRequest, NoteSuggestionCreate,
    NoteTrashBlocks, OptionalJsonValue,
};
use super::{
    assets, comments, data_source_board, data_source_buttons, data_source_calendar,
    data_source_gallery, data_source_list, data_source_rows, data_source_schema, data_source_table,
    data_source_templates, data_source_timeline, databases, history, local_user,
    mention_notifications, reads, suggestions, templates, undo_state, validation, writes,
};
use crate::db::run_migrations;
use serde_json::json;
use sqlx::{Row, SqlitePool};

const PAGE_A: &str = "11111111-1111-4111-8111-111111111111";
const PAGE_B: &str = "22222222-2222-4222-8222-222222222222";
const PAGE_C: &str = "33333333-3333-4333-8333-333333333333";
const BLOCK_A: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const BLOCK_B: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const BLOCK_C: &str = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const BLOCK_D: &str = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";
const BLOCK_E: &str = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee";
const BLOCK_F: &str = "ffffffff-ffff-4fff-8fff-ffffffffffff";
const COMMENT_A: &str = "10101010-1010-4010-8010-101010101010";
const COMMENT_B: &str = "20202020-2020-4020-8020-202020202020";
const COMMENT_C: &str = "30303030-3030-4030-8030-303030303030";
const SUGGESTION_A: &str = "40404040-4040-4040-8040-404040404040";
const SUGGESTION_B: &str = "50505050-5050-4050-8050-505050505050";
const TEMPLATE_A: &str = "90909090-9090-4090-8090-909090909090";
const TEMPLATE_B: &str = "91919191-9191-4191-8191-919191919191";
const DATABASE_A: &str = "80808080-8080-4080-8080-808080808080";
const DATA_SOURCE_A: &str = "81818181-8181-4181-8181-818181818181";
const DATABASE_VIEW_A: &str = "82828282-8282-4282-8282-828282828282";
const LINKED_DATABASE_A: &str = "83838383-8383-4383-8383-838383838383";
const LINKED_DATABASE_VIEW_A: &str = "84848484-8484-4484-8484-848484848484";
const DATABASE_B: &str = "85858585-8585-4585-8585-858585858585";
const DATA_SOURCE_B: &str = "86868686-8686-4686-8686-868686868686";
const DATABASE_VIEW_B: &str = "87878787-8787-4787-8787-878787878787";

async fn migrated_memory_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .unwrap();
    run_migrations(&pool).await.unwrap();
    pool
}

fn workspace_parent() -> NoteParent {
    NoteParent::Workspace { workspace: true }
}

fn page_parent(page_id: &str) -> NoteParent {
    NoteParent::PageId {
        page_id: page_id.to_string(),
    }
}

fn block_parent(block_id: &str) -> NoteParent {
    NoteParent::BlockId {
        block_id: block_id.to_string(),
    }
}

fn rich_text(content: &str) -> serde_json::Value {
    json!({
        "type": "text",
        "text": {
            "content": content,
            "link": null
        },
        "annotations": {
            "bold": false,
            "italic": false,
            "strikethrough": false,
            "underline": false,
            "code": false,
            "color": "default"
        },
        "plain_text": content,
        "href": null
    })
}

fn linked_rich_text(content: &str, url: &str) -> serde_json::Value {
    json!({
        "type": "text",
        "text": {
            "content": content,
            "link": {
                "url": url
            }
        },
        "annotations": {
            "bold": false,
            "italic": false,
            "strikethrough": false,
            "underline": false,
            "code": false,
            "color": "default"
        },
        "plain_text": content,
        "href": url
    })
}

fn annotated_rich_text(content: &str, color: &str) -> serde_json::Value {
    json!({
        "type": "text",
        "text": {
            "content": content,
            "link": null
        },
        "annotations": {
            "bold": true,
            "italic": true,
            "strikethrough": false,
            "underline": true,
            "code": false,
            "color": color
        },
        "plain_text": content,
        "href": null
    })
}

fn inline_equation_rich_text(expression: &str) -> serde_json::Value {
    json!({
        "type": "equation",
        "equation": {
            "expression": expression
        },
        "annotations": {
            "bold": false,
            "italic": false,
            "strikethrough": false,
            "underline": false,
            "code": false,
            "color": "default"
        },
        "plain_text": expression,
        "href": null
    })
}

fn page_mention(page_id: &str, title: &str) -> serde_json::Value {
    json!({
        "type": "mention",
        "mention": {
            "type": "page",
            "page": {
                "id": page_id
            }
        },
        "annotations": {
            "bold": false,
            "italic": false,
            "strikethrough": false,
            "underline": false,
            "code": false,
            "color": "default"
        },
        "plain_text": title,
        "href": format!("http://localhost:1420/?view=notes#notes?page={page_id}")
    })
}

fn date_mention(start: &str, title: &str, reminder: bool) -> serde_json::Value {
    let mut date = json!({
        "start": start,
        "end": null,
        "time_zone": null
    });
    if reminder {
        date["ganbaru_reminder"] = json!({ "enabled": true });
    }
    json!({
        "type": "mention",
        "mention": {
            "type": "date",
            "date": date
        },
        "annotations": {
            "bold": false,
            "italic": false,
            "strikethrough": false,
            "underline": false,
            "code": false,
            "color": "default"
        },
        "plain_text": title,
        "href": null
    })
}

fn user_mention(user_id: &str, title: &str) -> serde_json::Value {
    json!({
        "type": "mention",
        "mention": {
            "type": "user",
            "user": {
                "object": "user",
                "id": user_id
            }
        },
        "annotations": {
            "bold": false,
            "italic": false,
            "strikethrough": false,
            "underline": false,
            "code": false,
            "color": "default"
        },
        "plain_text": title,
        "href": null
    })
}

fn project_task_mention(task_id: &str, title: &str) -> serde_json::Value {
    json!({
        "type": "mention",
        "mention": {
            "type": "ganbaru_object",
            "ganbaru_object": {
                "type": "project_task",
                "id": task_id
            }
        },
        "annotations": {
            "bold": false,
            "italic": false,
            "strikethrough": false,
            "underline": false,
            "code": false,
            "color": "default"
        },
        "plain_text": title,
        "href": null
    })
}

fn paragraph_payload(content: &str) -> serde_json::Value {
    json!({
        "rich_text": [rich_text(content)],
        "color": "default"
    })
}

fn paragraph_icon_payload(content: &str) -> serde_json::Value {
    json!({
        "rich_text": [rich_text(content)],
        "color": "default",
        "icon": {
            "type": "icon",
            "icon": {
                "name": "star",
                "color": "yellow"
            }
        }
    })
}

fn heading_payload(content: &str, is_toggleable: bool, open: Option<bool>) -> serde_json::Value {
    let mut payload = paragraph_payload(content);
    payload["is_toggleable"] = json!(is_toggleable);
    if let Some(open) = open {
        payload["ganbaru_open"] = json!(open);
    }
    payload
}

fn todo_payload(content: &str, checked: bool) -> serde_json::Value {
    json!({
        "rich_text": [rich_text(content)],
        "checked": checked,
        "color": "default"
    })
}

fn code_payload(content: &str, language: &str) -> serde_json::Value {
    json!({
        "rich_text": [rich_text(content)],
        "caption": [],
        "language": language
    })
}

fn bookmark_payload(url: &str, caption: &str) -> serde_json::Value {
    let caption_items = if caption.is_empty() {
        Vec::<serde_json::Value>::new()
    } else {
        vec![rich_text(caption)]
    };
    json!({
        "caption": caption_items,
        "url": url
    })
}

fn embed_payload(url: &str) -> serde_json::Value {
    json!({
        "url": url
    })
}

fn link_preview_payload(url: &str) -> serde_json::Value {
    json!({
        "url": url
    })
}

fn synced_block_payload_original() -> serde_json::Value {
    json!({
        "synced_from": null
    })
}

fn synced_block_payload_duplicate(block_id: &str) -> serde_json::Value {
    json!({
        "synced_from": {
            "type": "block_id",
            "block_id": block_id
        }
    })
}

fn child_database_payload(title: &str) -> serde_json::Value {
    json!({
        "title": title
    })
}

fn template_payload(title: &str) -> serde_json::Value {
    json!({
        "rich_text": [rich_text(title)]
    })
}

fn button_payload(title: &str) -> serde_json::Value {
    json!({
        "rich_text": [rich_text(title)],
        "icon": {
            "type": "icon",
            "icon": {
                "name": "mouse-pointer-click",
                "color": "gray"
            }
        },
        "actions": [{
            "type": "insert_blocks",
            "source": "children",
            "position": "below_button"
        }]
    })
}

fn equation_payload(expression: &str) -> serde_json::Value {
    json!({
        "expression": expression
    })
}

fn unsupported_payload(block_type: &str) -> serde_json::Value {
    json!({
        "block_type": block_type,
        "source_type": "notion",
        "raw": {
            "type": "unsupported",
            "unsupported": {
                "block_type": block_type
            }
        },
        "warnings": ["Content is not exposed by the source API"]
    })
}

fn media_payload(url: &str, caption: &str, name: Option<&str>) -> serde_json::Value {
    let caption_items = if caption.is_empty() {
        Vec::<serde_json::Value>::new()
    } else {
        vec![rich_text(caption)]
    };
    let mut payload = json!({
        "caption": caption_items,
        "type": "external",
        "external": {
            "url": url
        }
    });
    if let Some(name) = name {
        payload["name"] = json!(name);
    }
    payload
}

fn local_media_payload(
    asset_path: &str,
    content_type: &str,
    byte_size: i64,
    sha256: &str,
    caption: &str,
    name: Option<&str>,
) -> serde_json::Value {
    let caption_items = if caption.is_empty() {
        Vec::<serde_json::Value>::new()
    } else {
        vec![rich_text(caption)]
    };
    let mut payload = json!({
        "caption": caption_items,
        "type": "file",
        "file": {
            "url": format!("ganbaru-asset:{asset_path}"),
            "content_type": content_type,
            "byte_size": byte_size,
            "sha256": sha256,
            "ganbaru_asset_path": asset_path
        }
    });
    if let Some(name) = name {
        payload["name"] = json!(name);
        payload["file"]["name"] = json!(name);
    }
    payload
}

fn table_payload(width: i64) -> serde_json::Value {
    json!({
        "table_width": width,
        "has_column_header": false,
        "has_row_header": false
    })
}

fn column_payload(width_ratio: Option<f64>) -> serde_json::Value {
    match width_ratio {
        Some(width_ratio) => json!({ "width_ratio": width_ratio }),
        None => json!({}),
    }
}

fn table_row_payload(cells: &[&str]) -> serde_json::Value {
    json!({
        "cells": cells
            .iter()
            .map(|cell| {
                if cell.is_empty() {
                    Vec::<serde_json::Value>::new()
                } else {
                    vec![rich_text(cell)]
                }
            })
            .collect::<Vec<_>>()
    })
}

fn tab_payload() -> serde_json::Value {
    json!({})
}

fn block(id: &str, block_type: &str, payload: serde_json::Value) -> NoteBlockWrite {
    NoteBlockWrite {
        id: id.to_string(),
        block_type: block_type.to_string(),
        paragraph: (block_type == "paragraph").then_some(payload.clone()),
        heading_1: (block_type == "heading_1").then_some(payload.clone()),
        heading_2: (block_type == "heading_2").then_some(payload.clone()),
        heading_3: (block_type == "heading_3").then_some(payload.clone()),
        heading_4: (block_type == "heading_4").then_some(payload.clone()),
        bulleted_list_item: (block_type == "bulleted_list_item").then_some(payload.clone()),
        numbered_list_item: (block_type == "numbered_list_item").then_some(payload.clone()),
        to_do: (block_type == "to_do").then_some(payload.clone()),
        toggle: (block_type == "toggle").then_some(payload.clone()),
        callout: (block_type == "callout").then_some(payload.clone()),
        quote: (block_type == "quote").then_some(payload.clone()),
        child_page: (block_type == "child_page").then_some(payload.clone()),
        child_database: (block_type == "child_database").then_some(payload.clone()),
        breadcrumb: (block_type == "breadcrumb").then_some(payload.clone()),
        table_of_contents: (block_type == "table_of_contents").then_some(payload.clone()),
        column_list: (block_type == "column_list").then_some(payload.clone()),
        column: (block_type == "column").then_some(payload.clone()),
        table: (block_type == "table").then_some(payload.clone()),
        table_row: (block_type == "table_row").then_some(payload.clone()),
        tab: (block_type == "tab").then_some(payload.clone()),
        image: (block_type == "image").then_some(payload.clone()),
        video: (block_type == "video").then_some(payload.clone()),
        audio: (block_type == "audio").then_some(payload.clone()),
        file: (block_type == "file").then_some(payload.clone()),
        pdf: (block_type == "pdf").then_some(payload.clone()),
        bookmark: (block_type == "bookmark").then_some(payload.clone()),
        link_preview: (block_type == "link_preview").then_some(payload.clone()),
        synced_block: (block_type == "synced_block").then_some(payload.clone()),
        template: (block_type == "template").then_some(payload.clone()),
        button: (block_type == "button").then_some(payload.clone()),
        embed: (block_type == "embed").then_some(payload.clone()),
        equation: (block_type == "equation").then_some(payload.clone()),
        divider: (block_type == "divider").then_some(payload.clone()),
        code: (block_type == "code").then_some(payload.clone()),
        unsupported: (block_type == "unsupported").then_some(payload),
    }
}

fn block_update(block_type: &str, payload: serde_json::Value) -> NoteBlockUpdate {
    NoteBlockUpdate {
        block_type: Some(block_type.to_string()),
        paragraph: (block_type == "paragraph").then_some(payload.clone()),
        heading_1: (block_type == "heading_1").then_some(payload.clone()),
        heading_2: (block_type == "heading_2").then_some(payload.clone()),
        heading_3: (block_type == "heading_3").then_some(payload.clone()),
        heading_4: (block_type == "heading_4").then_some(payload.clone()),
        bulleted_list_item: (block_type == "bulleted_list_item").then_some(payload.clone()),
        numbered_list_item: (block_type == "numbered_list_item").then_some(payload.clone()),
        to_do: (block_type == "to_do").then_some(payload.clone()),
        toggle: (block_type == "toggle").then_some(payload.clone()),
        callout: (block_type == "callout").then_some(payload.clone()),
        quote: (block_type == "quote").then_some(payload.clone()),
        child_page: (block_type == "child_page").then_some(payload.clone()),
        child_database: (block_type == "child_database").then_some(payload.clone()),
        breadcrumb: (block_type == "breadcrumb").then_some(payload.clone()),
        table_of_contents: (block_type == "table_of_contents").then_some(payload.clone()),
        column_list: (block_type == "column_list").then_some(payload.clone()),
        column: (block_type == "column").then_some(payload.clone()),
        table: (block_type == "table").then_some(payload.clone()),
        table_row: (block_type == "table_row").then_some(payload.clone()),
        tab: (block_type == "tab").then_some(payload.clone()),
        image: (block_type == "image").then_some(payload.clone()),
        video: (block_type == "video").then_some(payload.clone()),
        audio: (block_type == "audio").then_some(payload.clone()),
        file: (block_type == "file").then_some(payload.clone()),
        pdf: (block_type == "pdf").then_some(payload.clone()),
        bookmark: (block_type == "bookmark").then_some(payload.clone()),
        link_preview: (block_type == "link_preview").then_some(payload.clone()),
        synced_block: (block_type == "synced_block").then_some(payload.clone()),
        template: (block_type == "template").then_some(payload.clone()),
        button: (block_type == "button").then_some(payload.clone()),
        embed: (block_type == "embed").then_some(payload.clone()),
        equation: (block_type == "equation").then_some(payload.clone()),
        divider: (block_type == "divider").then_some(payload.clone()),
        code: (block_type == "code").then_some(payload.clone()),
        unsupported: (block_type == "unsupported").then_some(payload),
    }
}

async fn create_page(pool: &SqlitePool, page_id: &str, block_id: &str) {
    writes::create_page(
        pool,
        NotePageCreate {
            id: page_id.to_string(),
            title: "First page".to_string(),
            parent: workspace_parent(),
            first_block_id: block_id.to_string(),
            after_block_id: None,
        },
    )
    .await
    .unwrap();
}

async fn create_database(
    pool: &SqlitePool,
    database_id: &str,
    data_source_id: &str,
    view_id: &str,
    title: &str,
    after_block_id: &str,
) {
    databases::create_database(
        pool,
        NoteDatabaseCreate {
            id: database_id.to_string(),
            data_source_id: data_source_id.to_string(),
            view_id: view_id.to_string(),
            title: title.to_string(),
            parent: Some(page_parent(PAGE_A)),
            after_block_id: Some(after_block_id.to_string()),
            replace_block_id: None,
            icon: None,
            cover: None,
        },
    )
    .await
    .unwrap();
}

#[test]
fn notes_validation_rejects_bad_ids_and_payloads() {
    assert_eq!(
        validation::require_uuid("bad", "id"),
        Err("id must be a UUID".to_string())
    );
    assert_eq!(
        validation::validate_parent(&NoteParent::Workspace { workspace: false }),
        Err("workspace parent must set workspace to true".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("paragraph", &json!({ "rich_text": [{}] })),
        Err("rich text item type must be a string".to_string())
    );
    assert!(validation::validate_block_payload(
        "paragraph",
        &json!({ "rich_text": [page_mention(PAGE_B, "Target page")], "color": "default" }),
    )
    .is_ok());
    assert!(
        validation::validate_block_payload("paragraph", &paragraph_icon_payload("Overview"),)
            .is_ok()
    );
    assert!(validation::validate_block_payload(
        "paragraph",
        &json!({ "rich_text": [linked_rich_text("Docs", "https://example.com/docs")], "color": "default" }),
    )
    .is_ok());
    assert!(validation::validate_block_payload(
        "paragraph",
        &json!({ "rich_text": [linked_rich_text("Email", "mailto:team@example.com")], "color": "default" }),
    )
    .is_ok());
    assert!(validation::validate_block_payload(
        "paragraph",
        &json!({ "rich_text": [annotated_rich_text("Important", "blue_background")], "color": "default" }),
    )
    .is_ok());
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({ "rich_text": [annotated_rich_text("Bad", "rainbow")], "color": "default" }),
        ),
        Err("rich text annotations.color must be a supported Notion color".to_string())
    );
    assert!(validation::validate_block_payload(
        "paragraph",
        &json!({ "rich_text": [inline_equation_rich_text("\\frac{a}{b}")], "color": "default" }),
    )
    .is_ok());
    assert!(validation::validate_block_payload(
        "heading_1",
        &heading_payload("Toggle heading", true, Some(false)),
    )
    .is_ok());
    assert!(validation::validate_block_payload(
        "heading_4",
        &heading_payload("Small toggle heading", true, Some(false)),
    )
    .is_ok());
    assert_eq!(
        validation::validate_block_payload(
            "heading_1",
            &json!({
                "rich_text": [rich_text("Bad heading")],
                "color": "default",
                "is_toggleable": "yes"
            }),
        ),
        Err("heading_1.is_toggleable must be a boolean".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({ "rich_text": [inline_equation_rich_text("bad\u{0008}")], "color": "default" }),
        ),
        Err(
            "rich text equation.expression must not be empty or contain control characters"
                .to_string()
        )
    );
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({ "rich_text": [linked_rich_text("Bad", "javascript:alert(1)")], "color": "default" }),
        ),
        Err("rich text text.link.url must be a valid HTTP, HTTPS, or email URL".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({ "rich_text": [linked_rich_text("Bad", "mailto:team@example")], "color": "default" }),
        ),
        Err("rich text text.link.url must be a valid HTTP, HTTPS, or email URL".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({ "rich_text": [linked_rich_text("Bad", "mailto:team%40example.com")], "color": "default" }),
        ),
        Err("rich text text.link.url must be a valid HTTP, HTTPS, or email URL".to_string())
    );
    assert!(validation::validate_block_payload(
        "paragraph",
        &json!({ "rich_text": [date_mention("2026-06-30", "Remind Today", true)], "color": "default" }),
    )
    .is_ok());
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({ "rich_text": [date_mention("2026-99-30", "Bad date", false)], "color": "default" }),
        ),
        Err("rich text mention.date.start must be an ISO date or date-time".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({
                "rich_text": [{
                    "type": "mention",
                    "mention": {
                        "type": "database",
                        "database": { "id": PAGE_B }
                    },
                    "annotations": {
                        "bold": false,
                        "italic": false,
                        "strikethrough": false,
                        "underline": false,
                        "code": false,
                        "color": "default"
                    },
                    "plain_text": "Target database",
                    "href": null
                }],
                "color": "default"
            }),
        ),
        Ok(())
    );
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({
                "rich_text": [{
                    "type": "mention",
                    "mention": {
                        "type": "ganbaru_object",
                        "ganbaru_object": {
                            "type": "project_task",
                            "id": PAGE_B
                        }
                    },
                    "annotations": {
                        "bold": false,
                        "italic": false,
                        "strikethrough": false,
                        "underline": false,
                        "code": false,
                        "color": "default"
                    },
                    "plain_text": "Target task",
                    "href": null
                }],
                "color": "default"
            }),
        ),
        Ok(())
    );
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({
                "rich_text": [{
                    "type": "mention",
                    "mention": {
                        "type": "link_preview",
                        "link_preview": { "url": "https://example.com" }
                    },
                    "annotations": {
                        "bold": false,
                        "italic": false,
                        "strikethrough": false,
                        "underline": false,
                        "code": false,
                        "color": "default"
                    },
                    "plain_text": "Example",
                    "href": null
                }],
                "color": "default"
            }),
        ),
        Err("rich text mention.type is unsupported".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("nope", &json!({})),
        Err("unsupported block type: nope".to_string())
    );
    for block_type in ["meeting_notes", "transcription"] {
        assert_eq!(
            validation::validate_block_type(block_type),
            Err(format!(
                "{block_type} is blocked by the Notes block catalog gate until rich editor P0 completion, current block quality completion, honest docs, usable editing UI, persistence, focused tests, pnpm -w run validate are complete"
            ))
        );
        assert_eq!(
            validation::validate_block_payload(block_type, &json!({})),
            Err(format!(
                "{block_type} is blocked by the Notes block catalog gate until rich editor P0 completion, current block quality completion, honest docs, usable editing UI, persistence, focused tests, pnpm -w run validate are complete"
            ))
        );
    }
    assert_eq!(
        validation::validate_block_payload(
            "paragraph",
            &json!({ "rich_text": [rich_text("Bad")], "color": "neon" }),
        ),
        Err("paragraph.color must be a supported Notion color".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "toggle",
            &json!({ "rich_text": [rich_text("Bad")], "color": "default", "ganbaru_open": "yes" }),
        ),
        Err("toggle.ganbaru_open must be a boolean".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "callout",
            &json!({
                "rich_text": [rich_text("Bad")],
                "color": "default",
                "icon": { "type": "icon", "icon": { "name": "" } }
            }),
        ),
        Err("callout.icon.icon.name must be a non-empty string".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("table_of_contents", &json!({ "color": "neon" })),
        Err("table_of_contents.color must be a supported Notion color".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("child_page", &json!({ "title": 42 })),
        Err("child_page.title must be a string".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("child_page", &json!({ "title": "bad\u{0008}" })),
        Err("child_page.title must not contain control characters".to_string())
    );
    assert!(validation::validate_block_payload(
        "child_database",
        &json!({
            "title": "Tasks",
            "database_id": DATABASE_A,
            "data_source_id": DATA_SOURCE_A,
            "view_id": DATABASE_VIEW_A
        }),
    )
    .is_ok());
    assert_eq!(
        validation::validate_block_payload(
            "child_database",
            &json!({ "title": "Tasks", "database_id": "not-a-uuid" }),
        ),
        Err("child_database.database_id must be a UUID".to_string())
    );
    assert!(validation::validate_block_payload("template", &template_payload("Add task")).is_ok());
    assert_eq!(
        validation::validate_block_payload("template", &json!({ "rich_text": "Add task" })),
        Err("template.rich_text must be an array".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "template",
            &json!({ "rich_text": [rich_text("Add task")], "color": "default" }),
        ),
        Err("template.color is not supported".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "template",
            &json!({ "rich_text": [rich_text("Add task")], "children": [] }),
        ),
        Err("template.children must be stored as child blocks".to_string())
    );
    assert!(validation::validate_block_payload("button", &button_payload("Add agenda")).is_ok());
    for position in [
        "below_button",
        "above_button",
        "top_of_page",
        "bottom_of_page",
    ] {
        assert!(validation::validate_block_payload(
            "button",
            &json!({
                "rich_text": [rich_text("Add agenda")],
                "icon": null,
                "actions": [{
                    "type": "insert_blocks",
                    "source": "children",
                    "position": position
                }]
            }),
        )
        .is_ok());
    }
    assert_eq!(
        validation::validate_block_payload(
            "button",
            &json!({
                "rich_text": [rich_text("Add agenda")],
                "icon": null,
                "actions": []
            }),
        ),
        Err("button.actions must include between 1 and 10 actions".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "button",
            &json!({
                "rich_text": [rich_text("Add agenda")],
                "icon": null,
                "actions": [{
                    "type": "send_webhook",
                    "source": "children",
                    "position": "below_button"
                }]
            }),
        ),
        Err("button.actions[0].type must be insert_blocks".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "button",
            &json!({
                "rich_text": [rich_text("Add agenda")],
                "icon": null,
                "actions": [{
                    "type": "insert_blocks",
                    "source": "children",
                    "position": "nearby_database"
                }]
            }),
        ),
        Err("button.actions[0].position must be a supported button insert position".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "button",
            &json!({
                "rich_text": [rich_text("Add agenda")],
                "actions": [{
                    "type": "insert_blocks",
                    "source": "children",
                    "position": "below_button"
                }]
            }),
        ),
        Err("button.icon is required".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "button",
            &json!({
                "rich_text": [rich_text("Add agenda")],
                "icon": null,
                "children": [],
                "actions": [{
                    "type": "insert_blocks",
                    "source": "children",
                    "position": "below_button"
                }]
            }),
        ),
        Err("button.children must be stored as child blocks".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("child_database", &json!({ "title": 42 })),
        Err("child_database.title must be a string".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("child_database", &json!({ "title": "bad\u{0008}" }),),
        Err("child_database.title must not contain control characters".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("column", &json!({ "width_ratio": 0.0 })),
        Err("column.width_ratio must be greater than 0 and no more than 1".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "table",
            &json!({
                "table_width": 0,
                "has_column_header": false,
                "has_row_header": false
            }),
        ),
        Err("table.table_width must be between 1 and 100".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "table",
            &json!({
                "table_width": 2,
                "has_column_header": "yes",
                "has_row_header": false
            }),
        ),
        Err("table.has_column_header must be a boolean".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("table_row", &json!({ "cells": [{}] })),
        Err("table_row.cells entries must be rich text arrays".to_string())
    );
    assert!(validation::validate_block_payload("tab", &tab_payload()).is_ok());
    assert_eq!(
        validation::validate_block_payload("tab", &json!({ "title": "Bad" })),
        Err("tab must be an empty object".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "bookmark",
            &json!({ "caption": [{}], "url": "https://example.com" }),
        ),
        Err("rich text item type must be a string".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "bookmark",
            &json!({ "caption": [], "url": "bad\u{0008}url" }),
        ),
        Err("bookmark.url must not contain control characters".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("equation", &json!({ "expression": 42 })),
        Err("equation.expression must be a string".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("embed", &json!({ "url": 42 })),
        Err("embed.url must be a string".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("embed", &json!({ "url": "bad\u{0008}url" })),
        Err("embed.url must not contain control characters".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("link_preview", &json!({ "url": 42 })),
        Err("link_preview.url must be a string".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("link_preview", &json!({ "url": "bad\u{0008}url" }),),
        Err("link_preview.url must not contain control characters".to_string())
    );
    assert!(
        validation::validate_block_payload("synced_block", &synced_block_payload_original())
            .is_ok()
    );
    assert!(validation::validate_block_payload(
        "synced_block",
        &synced_block_payload_duplicate(BLOCK_A),
    )
    .is_ok());
    assert_eq!(
        validation::validate_block_payload("synced_block", &json!({})),
        Err("synced_block.synced_from is required".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("synced_block", &json!({ "synced_from": "bad" })),
        Err("synced_block.synced_from must be null or an object".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "synced_block",
            &json!({ "synced_from": { "type": "page_id", "block_id": BLOCK_A } }),
        ),
        Err("synced_block.synced_from.type must be block_id".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "synced_block",
            &json!({ "synced_from": { "type": "block_id", "block_id": "bad" } }),
        ),
        Err("synced_block.synced_from.block_id must be a UUID".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "image",
            &media_payload("http://example.com/image.png", "", None),
        ),
        Err("image.external.url must be a supported HTTPS image URL".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "pdf",
            &media_payload("https://example.com/file.txt", "", None),
        ),
        Err("pdf.external.url must be a supported HTTPS pdf URL".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "file",
            &json!({
                "caption": [],
                "type": "file_upload",
                "file_upload": { "id": "bad" }
            }),
        ),
        Err("file.file_upload.id must be a UUID".to_string())
    );
    assert!(validation::validate_block_payload(
        "image",
        &local_media_payload(
            "notes/files/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
            "image/png",
            42,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "",
            Some("image.png"),
        ),
    )
    .is_ok());
    assert_eq!(
        validation::validate_block_payload(
            "image",
            &local_media_payload(
                "notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
                "image/png",
                42,
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "",
                Some("image.png"),
            ),
        ),
        Err("image.file.ganbaru_asset_path must stay under the managed Notes file directory"
            .to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "pdf",
            &local_media_payload(
                "notes/files/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.pdf",
                "text/plain",
                42,
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "",
                Some("report.pdf"),
            ),
        ),
        Err("pdf.file.content_type must match the local media block type".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("equation", &json!({ "expression": "bad\u{0008}" })),
        Err("equation.expression must not contain control characters".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("unsupported", &json!({ "block_type": "" })),
        Err("unsupported.block_type must not be empty".to_string())
    );
    assert_eq!(
        validation::validate_block_payload(
            "unsupported",
            &json!({ "block_type": "bad\u{0008}type" }),
        ),
        Err("unsupported.block_type must not contain control characters".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("unsupported", &json!({ "raw": "form" })),
        Err("unsupported.raw must be an object".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("unsupported", &json!({ "warnings": [""] })),
        Err("unsupported.warnings[0] must not be empty".to_string())
    );
}

#[test]
fn undo_state_round_trips_and_clears_for_active_page() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        let state_json = r#"{"schema_version":1,"undo":[],"redo":[]}"#;

        undo_state::save_undo_state(&pool, PAGE_A, state_json)
            .await
            .unwrap();
        assert_eq!(
            undo_state::load_undo_state(&pool, PAGE_A).await.unwrap(),
            Some(state_json.to_string())
        );

        undo_state::clear_undo_state(&pool, PAGE_A).await.unwrap();
        assert_eq!(
            undo_state::load_undo_state(&pool, PAGE_A).await.unwrap(),
            None
        );
    });
}

#[test]
fn undo_state_rejects_invalid_or_oversized_json() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let invalid_error = undo_state::save_undo_state(&pool, PAGE_A, "{bad")
            .await
            .unwrap_err();
        assert!(invalid_error.starts_with("parse notes undo state:"));

        let oversized = format!(r#"{{"payload":"{}"}}"#, "x".repeat(512 * 1024));
        assert_eq!(
            undo_state::save_undo_state(&pool, PAGE_A, &oversized).await,
            Err("notes undo state is too large".to_string())
        );
    });
}

#[test]
fn undo_state_rejects_inactive_pages() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::trash_page(&pool, PAGE_A, true).await.unwrap();

        assert_eq!(
            undo_state::save_undo_state(&pool, PAGE_A, "{}").await,
            Err("notes page not found".to_string())
        );
    });
}

#[test]
fn create_page_persists_title_and_initial_paragraph() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let pages = reads::list_pages(&pool).await.unwrap();
        assert_eq!(pages.len(), 1);

        let block_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM notes_blocks WHERE page_id = ? AND type = 'paragraph'",
        )
        .bind(PAGE_A)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(block_count, 1);
    });
}

#[test]
fn create_nested_page_appends_child_page_block_to_parent_page() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested page".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: Some(BLOCK_A.to_string()),
            },
        )
        .await
        .unwrap();

        let parent_blocks = reads::get_block_children(&pool, PAGE_A, None, Some(10))
            .await
            .unwrap();
        let parent_json = serde_json::to_value(parent_blocks).unwrap();
        assert_eq!(parent_json["results"][1]["id"], PAGE_B);
        assert_eq!(parent_json["results"][1]["type"], "child_page");
        assert_eq!(
            parent_json["results"][1]["child_page"]["title"],
            "Nested page"
        );

        let child_blocks = reads::get_block_children(&pool, PAGE_B, None, Some(10))
            .await
            .unwrap();
        let child_json = serde_json::to_value(child_blocks).unwrap();
        assert_eq!(child_json["results"][0]["id"], BLOCK_B);
        assert_eq!(child_json["results"][0]["type"], "paragraph");
    });
}

#[test]
fn page_templates_create_apply_update_duplicate_and_delete() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: Some("Weekly review".to_string()),
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Unset,
                cover: OptionalJsonValue::Unset,
            },
        )
        .await
        .unwrap();
        writes::update_block(
            &pool,
            BLOCK_A,
            block_update("paragraph", paragraph_payload("Reflect on the week")),
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "to_do",
                    todo_payload("Choose next focus", false),
                )],
            },
        )
        .await
        .unwrap();

        let template = templates::create_page_template_from_page(
            &pool,
            NotePageTemplateCreateFromPage {
                id: TEMPLATE_A.to_string(),
                source_page_id: PAGE_A.to_string(),
                name: "Weekly review".to_string(),
            },
        )
        .await
        .unwrap();
        let template_json = serde_json::to_value(template).unwrap();
        assert_eq!(template_json["name"], "Weekly review");
        assert_eq!(template_json["block_count"], 2);
        assert!(template_json["properties"].get("title").is_some());

        let loaded = templates::apply_page_template(
            &pool,
            TEMPLATE_A,
            NotePageTemplateApply {
                parent: workspace_parent(),
                title: Some("Friday review".to_string()),
            },
        )
        .await
        .unwrap();
        let loaded_json = serde_json::to_value(loaded).unwrap();
        let applied_page_id = loaded_json["page"]["id"].as_str().unwrap().to_string();
        assert_eq!(
            loaded_json["page"]["properties"]["title"]["title"][0]["plain_text"],
            "Friday review"
        );
        assert_eq!(
            loaded_json["blocks"]["results"].as_array().unwrap().len(),
            2
        );
        assert_eq!(loaded_json["blocks"]["results"][0]["type"], "paragraph");
        assert_eq!(
            loaded_json["blocks"]["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Reflect on the week"
        );
        assert_eq!(loaded_json["blocks"]["results"][1]["type"], "to_do");

        create_page(&pool, PAGE_C, BLOCK_C).await;
        writes::update_page(
            &pool,
            PAGE_C,
            super::models::NotePageUpdate {
                title: Some("Daily plan".to_string()),
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Unset,
                cover: OptionalJsonValue::Unset,
            },
        )
        .await
        .unwrap();
        writes::update_block(
            &pool,
            BLOCK_C,
            block_update("paragraph", paragraph_payload("Plan the day")),
        )
        .await
        .unwrap();
        let updated = templates::update_page_template(
            &pool,
            TEMPLATE_A,
            NotePageTemplateUpdate {
                name: Some("Daily plan".to_string()),
                source_page_id: Some(PAGE_C.to_string()),
            },
        )
        .await
        .unwrap();
        let updated_json = serde_json::to_value(updated).unwrap();
        assert_eq!(updated_json["name"], "Daily plan");
        assert_eq!(updated_json["block_count"], 1);

        let duplicate = templates::duplicate_page_template(
            &pool,
            TEMPLATE_A,
            NotePageTemplateDuplicate {
                id: TEMPLATE_B.to_string(),
                name: "Daily plan copy".to_string(),
            },
        )
        .await
        .unwrap();
        let duplicate_json = serde_json::to_value(duplicate).unwrap();
        assert_eq!(duplicate_json["name"], "Daily plan copy");
        assert_eq!(duplicate_json["block_count"], 1);

        assert_eq!(
            templates::delete_page_template(&pool, TEMPLATE_A)
                .await
                .unwrap(),
            TEMPLATE_A
        );
        let templates = templates::list_page_templates(&pool).await.unwrap();
        let templates_json = serde_json::to_value(templates).unwrap();
        assert_eq!(templates_json.as_array().unwrap().len(), 1);
        assert_eq!(templates_json[0]["id"], TEMPLATE_B);

        let applied_page_exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM notes_pages WHERE id = ?")
                .bind(applied_page_id)
                .fetch_optional(&pool)
                .await
                .unwrap();
        assert_eq!(applied_page_exists, Some(1));
    });
}

#[test]
fn page_history_snapshots_restore_copy_and_retention_settings() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let settings = history::get_page_history_settings(&pool).await.unwrap();
        let settings_json = serde_json::to_value(settings).unwrap();
        assert_eq!(settings_json["retention_days"], 30);

        writes::update_block(
            &pool,
            BLOCK_A,
            block_update("paragraph", paragraph_payload("Draft one")),
        )
        .await
        .unwrap();
        let snapshots = history::list_page_history_snapshots(&pool, PAGE_A)
            .await
            .unwrap();
        let snapshots_json = serde_json::to_value(&snapshots).unwrap();
        let local_user_json =
            serde_json::to_value(local_user::get_local_user(&pool).await.unwrap()).unwrap();
        assert_eq!(snapshots_json.as_array().unwrap().len(), 1);
        assert_eq!(snapshots_json[0]["block_count"], 1);
        assert_eq!(snapshots_json[0]["created_by"]["id"], local_user_json["id"]);
        let initial_snapshot_id = snapshots_json[0]["id"].as_str().unwrap().to_string();

        sqlx::query(
            "UPDATE notes_page_history_snapshots
             SET created_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-10 minutes')
             WHERE id = ?",
        )
        .bind(&initial_snapshot_id)
        .execute(&pool)
        .await
        .unwrap();

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "to_do",
                    todo_payload("Review the draft", false),
                )],
            },
        )
        .await
        .unwrap();

        let snapshots = history::list_page_history_snapshots(&pool, PAGE_A)
            .await
            .unwrap();
        let snapshots_json = serde_json::to_value(&snapshots).unwrap();
        assert_eq!(snapshots_json.as_array().unwrap().len(), 2);
        let draft_snapshot_id = snapshots_json[0]["id"].as_str().unwrap().to_string();

        let initial_version =
            history::load_page_history_snapshot(&pool, PAGE_A, &initial_snapshot_id)
                .await
                .unwrap();
        let initial_json = serde_json::to_value(initial_version).unwrap();
        assert_eq!(
            initial_json["blocks"]["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            ""
        );

        let draft_version = history::load_page_history_snapshot(&pool, PAGE_A, &draft_snapshot_id)
            .await
            .unwrap();
        let draft_json = serde_json::to_value(draft_version).unwrap();
        assert_eq!(
            draft_json["blocks"]["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Draft one"
        );

        let copied = history::copy_page_history_blocks(
            &pool,
            PAGE_A,
            &draft_snapshot_id,
            NotePageHistoryCopyBlocks {
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        let copied_json = serde_json::to_value(copied).unwrap();
        assert_eq!(copied_json["results"].as_array().unwrap().len(), 1);
        assert_eq!(
            copied_json["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Draft one"
        );

        let restored = history::restore_page_history_snapshot(&pool, PAGE_A, &initial_snapshot_id)
            .await
            .unwrap();
        let restored_json = serde_json::to_value(restored).unwrap();
        assert_eq!(
            restored_json["blocks"]["results"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            restored_json["blocks"]["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            ""
        );

        let forever = history::update_page_history_settings(
            &pool,
            NotePageHistorySettingsUpdate {
                retention_days: None,
            },
        )
        .await
        .unwrap();
        let forever_json = serde_json::to_value(forever).unwrap();
        assert_eq!(forever_json["retention_days"], serde_json::Value::Null);

        let retained = history::update_page_history_settings(
            &pool,
            NotePageHistorySettingsUpdate {
                retention_days: Some(90),
            },
        )
        .await
        .unwrap();
        let retained_json = serde_json::to_value(retained).unwrap();
        assert_eq!(retained_json["retention_days"], 90);
    });
}

#[test]
fn page_history_coalesces_rapid_editor_snapshots() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::update_block(
            &pool,
            BLOCK_A,
            block_update("paragraph", paragraph_payload("First edit")),
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Second row"))],
            },
        )
        .await
        .unwrap();
        writes::update_block(
            &pool,
            BLOCK_B,
            block_update("paragraph", paragraph_payload("Second row edited")),
        )
        .await
        .unwrap();

        let snapshots = history::list_page_history_snapshots(&pool, PAGE_A)
            .await
            .unwrap();
        let snapshots_json = serde_json::to_value(snapshots).unwrap();
        assert_eq!(snapshots_json.as_array().unwrap().len(), 1);
        assert_eq!(snapshots_json[0]["reason"], "update_block");
        assert_eq!(snapshots_json[0]["block_count"], 1);
    });
}

#[test]
fn sidebar_pages_load_roots_expanded_children_and_selected_ancestors() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_C.to_string(),
                title: "Leaf".to_string(),
                parent: page_parent(PAGE_B),
                first_block_id: BLOCK_C.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();

        let roots = reads::list_sidebar_pages(
            &pool,
            NoteSidebarPagesRequest {
                expanded_page_ids: vec![],
                seed_page_ids: vec![],
                selected_page_id: None,
            },
        )
        .await
        .unwrap();
        let roots_json = serde_json::to_value(roots).unwrap();
        assert_eq!(roots_json["pages"].as_array().unwrap().len(), 1);
        assert_eq!(roots_json["pages"][0]["id"], PAGE_A);
        assert!(roots_json["pages"][0]["blocks"].is_null());
        assert_eq!(roots_json["page_ids_with_children"], json!([PAGE_A]));

        let expanded = reads::list_sidebar_pages(
            &pool,
            NoteSidebarPagesRequest {
                expanded_page_ids: vec![PAGE_A.to_string()],
                seed_page_ids: vec![],
                selected_page_id: None,
            },
        )
        .await
        .unwrap();
        let expanded_json = serde_json::to_value(expanded).unwrap();
        let expanded_ids = expanded_json["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|page| page["id"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(expanded_ids, vec![PAGE_B, PAGE_A]);
        assert_eq!(
            expanded_json["page_ids_with_children"],
            json!([PAGE_A, PAGE_B])
        );

        let selected = reads::list_sidebar_pages(
            &pool,
            NoteSidebarPagesRequest {
                expanded_page_ids: vec![],
                seed_page_ids: vec![],
                selected_page_id: Some(PAGE_C.to_string()),
            },
        )
        .await
        .unwrap();
        let selected_json = serde_json::to_value(selected).unwrap();
        let selected_ids = selected_json["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|page| page["id"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(selected_ids.len(), 3);
        assert!(selected_ids.contains(&PAGE_A));
        assert!(selected_ids.contains(&PAGE_B));
        assert!(selected_ids.contains(&PAGE_C));
    });
}

#[test]
fn sidebar_pages_report_trashed_parents_for_seed_pages() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        sqlx::query("UPDATE notes_pages SET in_trash = 1 WHERE id = ?")
            .bind(PAGE_A)
            .execute(&pool)
            .await
            .unwrap();

        let sidebar_pages = reads::list_sidebar_pages(
            &pool,
            NoteSidebarPagesRequest {
                expanded_page_ids: vec![],
                seed_page_ids: vec![PAGE_B.to_string()],
                selected_page_id: None,
            },
        )
        .await
        .unwrap();
        let sidebar_json = serde_json::to_value(sidebar_pages).unwrap();
        assert_eq!(sidebar_json["pages"].as_array().unwrap().len(), 1);
        assert_eq!(sidebar_json["pages"][0]["id"], PAGE_B);
        assert_eq!(sidebar_json["trashed_parent_page_ids"], json!([PAGE_A]));
    });
}

#[test]
fn create_nested_page_can_insert_after_block_parent_sibling() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_A),
                after: None,
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Nested"))],
            },
        )
        .await
        .unwrap();

        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested under block".to_string(),
                parent: block_parent(BLOCK_A),
                first_block_id: BLOCK_D.to_string(),
                after_block_id: Some(BLOCK_B.to_string()),
            },
        )
        .await
        .unwrap();

        let children = reads::get_block_children(&pool, BLOCK_A, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        assert_eq!(children_json["results"][0]["id"], BLOCK_B);
        assert_eq!(children_json["results"][1]["id"], PAGE_B);
        assert_eq!(children_json["results"][1]["type"], "child_page");
        assert_eq!(
            children_json["results"][1]["child_page"]["title"],
            "Nested under block"
        );

        let child_page = reads::get_page(&pool, PAGE_B, false).await.unwrap();
        let child_page_json = serde_json::to_value(child_page).unwrap();
        assert_eq!(child_page_json["parent"]["block_id"], BLOCK_A);
    });
}

#[test]
fn create_local_database_inside_notes_page() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let created = databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: Some(json!({
                    "type": "icon",
                    "icon": {
                        "name": "table",
                        "color": "blue"
                    }
                })),
                cover: None,
            },
        )
        .await
        .unwrap();

        let created_json = serde_json::to_value(created).unwrap();
        assert_eq!(created_json["database"]["id"], DATABASE_A);
        assert_eq!(created_json["database"]["parent"]["page_id"], PAGE_A);
        assert_eq!(
            created_json["database"]["data_sources"][0]["id"],
            DATA_SOURCE_A
        );
        assert_eq!(
            created_json["data_source"]["parent"]["database_id"],
            DATABASE_A
        );
        assert_eq!(created_json["view"]["type"], "table");
        assert_eq!(created_json["block"]["type"], "child_database");
        assert_eq!(
            created_json["block"]["child_database"]["database_id"],
            DATABASE_A
        );

        let properties: String =
            sqlx::query_scalar("SELECT properties FROM notes_data_sources WHERE id = ?")
                .bind(DATA_SOURCE_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        let properties_json: serde_json::Value = serde_json::from_str(&properties).unwrap();
        assert_eq!(properties_json["Name"]["type"], "title");

        let children = reads::get_block_children(&pool, PAGE_A, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        assert_eq!(children_json["results"][0]["id"], BLOCK_A);
        assert_eq!(children_json["results"][1]["id"], DATABASE_A);
        assert_eq!(children_json["results"][1]["type"], "child_database");
    });
}

#[test]
fn linked_database_view_shares_source_and_keeps_view_settings_independent() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "type": "rich_text",
                        "rich_text": {}
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "type": "number",
                        "number": { "format": "number" }
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "details".to_string(),
                    "estimate".to_string(),
                ],
                hidden_property_ids: vec!["details".to_string()],
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(DATABASE_A),
            Some(DATABASE_VIEW_A),
            NoteDataSourceTableViewUpdate {
                filter: vec![],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "estimate".to_string(),
                    direction: "descending".to_string(),
                }],
                configuration: NoteDataSourceTableConfigurationUpdate {
                    property_order: vec![
                        "title".to_string(),
                        "details".to_string(),
                        "estimate".to_string(),
                    ],
                    hidden_property_ids: vec!["details".to_string()],
                    column_widths: json!({ "title": 260, "details": 180, "estimate": 120 }),
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();

        let linked = databases::create_linked_database_view(
            &pool,
            NoteLinkedDatabaseCreate {
                id: LINKED_DATABASE_A.to_string(),
                view_id: LINKED_DATABASE_VIEW_A.to_string(),
                source_block_id: DATABASE_A.to_string(),
                title: Some("Task mirror".to_string()),
            },
        )
        .await
        .unwrap();
        let linked_json = serde_json::to_value(linked).unwrap();
        assert_eq!(linked_json["database"]["id"], LINKED_DATABASE_A);
        assert_eq!(
            linked_json["data_source"]["parent"]["database_id"],
            DATABASE_A
        );
        assert_eq!(
            linked_json["view"]["parent"]["database_id"],
            LINKED_DATABASE_A
        );
        assert_eq!(linked_json["view"]["data_source_id"], DATA_SOURCE_A);
        assert_eq!(
            linked_json["block"]["child_database"]["database_id"],
            LINKED_DATABASE_A
        );
        assert_eq!(
            linked_json["block"]["child_database"]["data_source_id"],
            DATA_SOURCE_A
        );

        let linked_data_source_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_data_sources WHERE database_id = ?")
                .bind(LINKED_DATABASE_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(linked_data_source_count, 0);

        data_source_table::update_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(LINKED_DATABASE_A),
            Some(LINKED_DATABASE_VIEW_A),
            NoteDataSourceTableViewUpdate {
                filter: vec![],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "title".to_string(),
                    direction: "ascending".to_string(),
                }],
                configuration: NoteDataSourceTableConfigurationUpdate {
                    property_order: vec![
                        "title".to_string(),
                        "estimate".to_string(),
                        "details".to_string(),
                    ],
                    hidden_property_ids: vec!["estimate".to_string()],
                    column_widths: json!({ "title": 320, "details": 180, "estimate": 120 }),
                    row_open_mode: "full_page".to_string(),
                },
            },
        )
        .await
        .unwrap();

        let source_table = data_source_table::get_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(DATABASE_A),
            Some(DATABASE_VIEW_A),
        )
        .await
        .unwrap();
        let source_json = serde_json::to_value(source_table).unwrap();
        assert_eq!(
            source_json["view"]["configuration"]["table"]["hidden_property_ids"],
            json!(["details"])
        );
        assert_eq!(
            source_json["view"]["configuration"]["table"]["row_open_mode"],
            "side_panel"
        );
        assert_eq!(source_json["view"]["sorts"][0]["property_id"], "estimate");

        let linked_table = data_source_table::get_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(LINKED_DATABASE_A),
            Some(LINKED_DATABASE_VIEW_A),
        )
        .await
        .unwrap();
        let linked_table_json = serde_json::to_value(linked_table).unwrap();
        assert_eq!(
            linked_table_json["view"]["configuration"]["table"]["hidden_property_ids"],
            json!(["estimate"])
        );
        assert_eq!(
            linked_table_json["view"]["configuration"]["table"]["row_open_mode"],
            "full_page"
        );
        assert_eq!(
            linked_table_json["view"]["sorts"][0]["property_id"],
            "title"
        );

        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            Some(LINKED_DATABASE_VIEW_A),
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "type": "rich_text",
                        "rich_text": {}
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "type": "number",
                        "number": { "format": "number" }
                    },
                    "Due": {
                        "id": "due",
                        "name": "Due",
                        "type": "date",
                        "date": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "estimate".to_string(),
                    "details".to_string(),
                    "due".to_string(),
                ],
                hidden_property_ids: vec!["estimate".to_string()],
            },
        )
        .await
        .unwrap();

        let reloaded_linked = data_source_table::get_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(LINKED_DATABASE_A),
            Some(LINKED_DATABASE_VIEW_A),
        )
        .await
        .unwrap();
        let reloaded_linked_json = serde_json::to_value(reloaded_linked).unwrap();
        assert_eq!(
            reloaded_linked_json["data_source"]["properties"]["Due"]["type"],
            "date"
        );
        assert_eq!(
            reloaded_linked_json["view"]["parent"]["database_id"],
            LINKED_DATABASE_A
        );
        assert_eq!(
            reloaded_linked_json["view"]["configuration"]["table"]["hidden_property_ids"],
            json!(["estimate"])
        );
    });
}

#[test]
fn database_row_pages_are_real_pages_with_page_lifecycle() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "type": "rich_text",
                        "rich_text": {}
                    },
                    "Done": {
                        "id": "done_checkbox",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "details".to_string(),
                    "done_checkbox".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();

        let loaded = data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Write spec".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        let loaded_json = serde_json::to_value(&loaded).unwrap();
        assert_eq!(loaded_json["page"]["parent"]["type"], "data_source_id");
        assert_eq!(
            loaded_json["page"]["parent"]["data_source_id"],
            DATA_SOURCE_A
        );
        assert_eq!(
            loaded_json["page"]["properties"]["Name"]["title"][0]["plain_text"],
            "Write spec"
        );
        assert_eq!(loaded_json["page"]["properties"]["Done"]["checkbox"], false);
        assert_eq!(loaded_json["blocks"]["results"][0]["type"], "paragraph");

        let sidebar_pages = reads::list_sidebar_pages(
            &pool,
            NoteSidebarPagesRequest {
                expanded_page_ids: vec![],
                seed_page_ids: vec![],
                selected_page_id: None,
            },
        )
        .await
        .unwrap();
        let sidebar_json = serde_json::to_value(sidebar_pages).unwrap();
        assert!(!sidebar_json["pages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["id"] == PAGE_B));

        let rows = data_source_rows::list_data_source_row_pages(&pool, DATA_SOURCE_A)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);

        let search_results = reads::search(&pool, "Write", Some(10)).await.unwrap();
        let search_json = serde_json::to_value(search_results).unwrap();
        assert!(search_json
            .as_array()
            .unwrap()
            .iter()
            .any(|result| result["page"]["id"] == PAGE_B));

        let duplicated = writes::duplicate_page(&pool, PAGE_B, NoteDuplicatePage { title: None })
            .await
            .unwrap();
        let duplicated_json = serde_json::to_value(&duplicated).unwrap();
        let duplicate_id = duplicated_json["page"]["id"].as_str().unwrap().to_string();
        assert_ne!(duplicate_id, PAGE_B);
        assert_eq!(
            duplicated_json["page"]["parent"]["data_source_id"],
            DATA_SOURCE_A
        );
        assert_eq!(
            duplicated_json["page"]["properties"]["Name"]["title"][0]["plain_text"],
            "Write spec"
        );

        writes::trash_page(&pool, PAGE_B, true).await.unwrap();
        let rows_after_trash = data_source_rows::list_data_source_row_pages(&pool, DATA_SOURCE_A)
            .await
            .unwrap();
        assert_eq!(rows_after_trash.len(), 1);
        let trashed = reads::list_trashed_pages(&pool).await.unwrap();
        let trashed_json = serde_json::to_value(trashed).unwrap();
        assert!(trashed_json
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["id"] == PAGE_B));

        let restored = writes::trash_page(&pool, PAGE_B, false).await.unwrap();
        let restored_json = serde_json::to_value(restored).unwrap();
        assert_eq!(restored_json["parent"]["data_source_id"], DATA_SOURCE_A);
        let rows_after_restore = data_source_rows::list_data_source_row_pages(&pool, DATA_SOURCE_A)
            .await
            .unwrap();
        assert_eq!(rows_after_restore.len(), 2);

        sqlx::query("UPDATE notes_data_sources SET in_trash = 1 WHERE id = ?")
            .bind(DATA_SOURCE_A)
            .execute(&pool)
            .await
            .unwrap();
        let hidden_rows = data_source_rows::list_data_source_row_pages(&pool, DATA_SOURCE_A)
            .await
            .unwrap();
        assert!(hidden_rows.is_empty());
        let hidden_search = reads::search(&pool, "Write", Some(10)).await.unwrap();
        let hidden_search_json = serde_json::to_value(hidden_search).unwrap();
        assert!(!hidden_search_json
            .as_array()
            .unwrap()
            .iter()
            .any(|result| result["page"]["parent"]["type"] == "data_source_id"));
    });
}

#[test]
fn editable_database_table_view_persists_cells_configuration_filters_and_sorts() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "type": "rich_text",
                        "rich_text": {}
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "type": "number",
                        "number": { "format": "number" }
                    },
                    "Priority": {
                        "id": "priority",
                        "name": "Priority",
                        "type": "select",
                        "select": {
                            "options": [
                                { "id": "low", "name": "Low", "color": "blue" },
                                { "id": "high", "name": "High", "color": "red" }
                            ]
                        }
                    },
                    "Done": {
                        "id": "done_checkbox",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "details".to_string(),
                    "estimate".to_string(),
                    "priority".to_string(),
                    "done_checkbox".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();

        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Beta".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Alpha".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();

        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "details".to_string(),
                value: json!("Write backend"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(5),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "priority".to_string(),
                value: json!("High"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "done_checkbox".to_string(),
                value: json!(true),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "details".to_string(),
                value: json!("Write frontend"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(2),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "priority".to_string(),
                value: json!("Low"),
            },
        )
        .await
        .unwrap();

        let table = data_source_table::update_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceTableViewUpdate {
                filter: vec![NoteDataSourceTableFilter {
                    property_id: "title".to_string(),
                    condition: "contains".to_string(),
                    value: Some(json!("a")),
                }],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "estimate".to_string(),
                    direction: "descending".to_string(),
                }],
                configuration: NoteDataSourceTableConfigurationUpdate {
                    property_order: vec![
                        "title".to_string(),
                        "estimate".to_string(),
                        "priority".to_string(),
                        "done_checkbox".to_string(),
                        "details".to_string(),
                    ],
                    hidden_property_ids: vec!["details".to_string()],
                    column_widths: json!({
                        "title": 260,
                        "estimate": 160,
                        "priority": 180,
                        "done_checkbox": 112,
                        "details": 240
                    }),
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();

        let table_json = serde_json::to_value(table).unwrap();
        assert_eq!(
            table_json["view"]["configuration"]["table"]["hidden_property_ids"],
            json!(["details"])
        );
        assert_eq!(
            table_json["view"]["configuration"]["table"]["column_widths"]["estimate"],
            160
        );
        assert_eq!(
            table_json["view"]["configuration"]["table"]["row_open_mode"],
            "side_panel"
        );
        assert_eq!(
            table_json["view"]["filter"]["filters"][0]["property_id"],
            "title"
        );
        assert_eq!(table_json["view"]["sorts"][0]["property_id"], "estimate");
        assert_eq!(table_json["rows"].as_array().unwrap().len(), 2);
        assert_eq!(table_json["rows"][0]["id"], PAGE_B);
        assert_eq!(table_json["rows"][0]["properties"]["Estimate"]["number"], 5);
        assert_eq!(
            table_json["rows"][0]["properties"]["Priority"]["select"]["name"],
            "High"
        );
        assert_eq!(
            table_json["rows"][0]["properties"]["Details"]["rich_text"][0]["plain_text"],
            "Write backend"
        );

        let filtered = data_source_table::update_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceTableViewUpdate {
                filter: vec![NoteDataSourceTableFilter {
                    property_id: "done_checkbox".to_string(),
                    condition: "checked".to_string(),
                    value: None,
                }],
                sorts: vec![],
                configuration: NoteDataSourceTableConfigurationUpdate {
                    property_order: vec![
                        "title".to_string(),
                        "estimate".to_string(),
                        "priority".to_string(),
                        "done_checkbox".to_string(),
                        "details".to_string(),
                    ],
                    hidden_property_ids: vec!["details".to_string()],
                    column_widths: json!({
                        "title": 260,
                        "estimate": 160,
                        "priority": 180,
                        "done_checkbox": 112,
                        "details": 240
                    }),
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let filtered_json = serde_json::to_value(filtered).unwrap();
        assert_eq!(filtered_json["rows"].as_array().unwrap().len(), 1);
        assert_eq!(filtered_json["rows"][0]["id"], PAGE_B);
    });
}

#[test]
fn database_relations_persist_links_backlinks_and_search() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_database(
            &pool,
            DATABASE_A,
            DATA_SOURCE_A,
            DATABASE_VIEW_A,
            "Tasks",
            BLOCK_A,
        )
        .await;
        create_database(
            &pool,
            DATABASE_B,
            DATA_SOURCE_B,
            DATABASE_VIEW_B,
            "Projects",
            DATABASE_A,
        )
        .await;
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Project": {
                        "id": "project_relation",
                        "name": "Project",
                        "type": "relation",
                        "relation": {
                            "data_source_id": DATA_SOURCE_B
                        }
                    }
                }),
                property_order: vec!["title".to_string(), "project_relation".to_string()],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_B,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Project Alpha".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Write relation tests".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();

        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "project_relation".to_string(),
                value: json!([PAGE_C]),
            },
        )
        .await
        .unwrap();

        let links: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT source_page_id, source_property_id, target_page_id, target_data_source_id
             FROM notes_data_source_relation_links",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            links,
            vec![(
                PAGE_B.to_string(),
                "project_relation".to_string(),
                PAGE_C.to_string(),
                DATA_SOURCE_B.to_string()
            )]
        );

        let table = data_source_table::get_data_source_table_view(&pool, DATA_SOURCE_A, None, None)
            .await
            .unwrap();
        let table_json = serde_json::to_value(table).unwrap();
        assert_eq!(
            table_json["rows"][0]["properties"]["Project"]["relation"][0]["title"],
            "Project Alpha"
        );

        let backlinks = reads::list_backlinks(&pool, PAGE_C).await.unwrap();
        let backlinks_json = serde_json::to_value(backlinks).unwrap();
        assert!(backlinks_json.as_array().unwrap().iter().any(|backlink| {
            backlink["reference_type"] == "database_relation"
                && backlink["source_page"]["id"] == PAGE_B
        }));

        let search_results = reads::search(&pool, "Project Alpha", Some(10))
            .await
            .unwrap();
        let search_json = serde_json::to_value(search_results).unwrap();
        assert!(search_json
            .as_array()
            .unwrap()
            .iter()
            .any(|result| { result["page"]["id"] == PAGE_B }));

        let invalid = data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "project_relation".to_string(),
                value: json!([PAGE_B]),
            },
        )
        .await;
        assert_eq!(
            invalid.err().unwrap(),
            "relation target page must belong to the configured data source"
        );
    });
}

#[test]
fn database_relations_sync_two_way_when_inverse_property_is_configured() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_database(
            &pool,
            DATABASE_A,
            DATA_SOURCE_A,
            DATABASE_VIEW_A,
            "Tasks",
            BLOCK_A,
        )
        .await;
        create_database(
            &pool,
            DATABASE_B,
            DATA_SOURCE_B,
            DATABASE_VIEW_B,
            "Projects",
            DATABASE_A,
        )
        .await;
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_B,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Tasks": {
                        "id": "tasks_relation",
                        "name": "Tasks",
                        "type": "relation",
                        "relation": {
                            "data_source_id": DATA_SOURCE_A
                        }
                    }
                }),
                property_order: vec!["title".to_string(), "tasks_relation".to_string()],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Project": {
                        "id": "project_relation",
                        "name": "Project",
                        "type": "relation",
                        "relation": {
                            "data_source_id": DATA_SOURCE_B,
                            "dual_property": {
                                "synced_property_id": "tasks_relation",
                                "synced_property_name": "Tasks"
                            }
                        }
                    }
                }),
                property_order: vec!["title".to_string(), "project_relation".to_string()],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_B,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Project Alpha".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Write relation tests".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();

        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "project_relation".to_string(),
                value: json!([PAGE_C]),
            },
        )
        .await
        .unwrap();

        let project = reads::get_page(&pool, PAGE_C, false).await.unwrap();
        let project_json = serde_json::to_value(project).unwrap();
        assert_eq!(
            project_json["properties"]["Tasks"]["relation"][0]["id"],
            PAGE_B
        );
        let link_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_data_source_relation_links")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(link_count, 2);

        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "project_relation".to_string(),
                value: json!([]),
            },
        )
        .await
        .unwrap();

        let project = reads::get_page(&pool, PAGE_C, false).await.unwrap();
        let project_json = serde_json::to_value(project).unwrap();
        assert_eq!(project_json["properties"]["Tasks"]["relation"], json!([]));
        let link_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_data_source_relation_links")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(link_count, 0);
    });
}

#[test]
fn database_rollups_compute_from_relations_and_invalidate_cache() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_database(
            &pool,
            DATABASE_A,
            DATA_SOURCE_A,
            DATABASE_VIEW_A,
            "Tasks",
            BLOCK_A,
        )
        .await;
        create_database(
            &pool,
            DATABASE_B,
            DATA_SOURCE_B,
            DATABASE_VIEW_B,
            "Projects",
            DATABASE_A,
        )
        .await;
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_B,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Budget": {
                        "id": "budget",
                        "name": "Budget",
                        "type": "number",
                        "number": {
                            "format": "number"
                        }
                    }
                }),
                property_order: vec!["title".to_string(), "budget".to_string()],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Project": {
                        "id": "project_relation",
                        "name": "Project",
                        "type": "relation",
                        "relation": {
                            "data_source_id": DATA_SOURCE_B
                        }
                    },
                    "Project budget": {
                        "id": "project_budget",
                        "name": "Project budget",
                        "type": "rollup",
                        "rollup": {
                            "relation_property_id": "project_relation",
                            "relation_property_name": "Project",
                            "rollup_property_id": "budget",
                            "rollup_property_name": "Budget",
                            "function": "sum"
                        }
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "project_relation".to_string(),
                    "project_budget".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_B,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Project Alpha".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_B,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "budget".to_string(),
                value: json!(7),
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Write rollup tests".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "project_relation".to_string(),
                value: json!([PAGE_C]),
            },
        )
        .await
        .unwrap();

        let table = data_source_table::get_data_source_table_view(&pool, DATA_SOURCE_A, None, None)
            .await
            .unwrap();
        let table_json = serde_json::to_value(table).unwrap();
        assert_eq!(
            table_json["rows"][0]["properties"]["Project budget"]["rollup"]["number"].as_f64(),
            Some(7.0)
        );
        let stored_properties: String =
            sqlx::query_scalar("SELECT properties FROM notes_pages WHERE id = ?")
                .bind(PAGE_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        let stored_json: serde_json::Value = serde_json::from_str(&stored_properties).unwrap();
        assert!(stored_json.get("Project budget").is_none());
        let cache_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_data_source_rollup_cache")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(cache_count, 1);

        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_B,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "budget".to_string(),
                value: json!(11),
            },
        )
        .await
        .unwrap();
        let cache_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_data_source_rollup_cache")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(cache_count, 0);

        let table = data_source_table::get_data_source_table_view(&pool, DATA_SOURCE_A, None, None)
            .await
            .unwrap();
        let table_json = serde_json::to_value(table).unwrap();
        assert_eq!(
            table_json["rows"][0]["properties"]["Project budget"]["rollup"]["number"].as_f64(),
            Some(11.0)
        );
    });
}

#[test]
fn database_rollups_reject_incompatible_schema_configuration() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_database(
            &pool,
            DATABASE_A,
            DATA_SOURCE_A,
            DATABASE_VIEW_A,
            "Tasks",
            BLOCK_A,
        )
        .await;
        create_database(
            &pool,
            DATABASE_B,
            DATA_SOURCE_B,
            DATABASE_VIEW_B,
            "Projects",
            DATABASE_A,
        )
        .await;
        let invalid = data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Project": {
                        "id": "project_relation",
                        "name": "Project",
                        "type": "relation",
                        "relation": {
                            "data_source_id": DATA_SOURCE_B
                        }
                    },
                    "Bad rollup": {
                        "id": "bad_rollup",
                        "name": "Bad rollup",
                        "type": "rollup",
                        "rollup": {
                            "relation_property_id": "project_relation",
                            "relation_property_name": "Project",
                            "rollup_property_id": "title",
                            "rollup_property_name": "Name",
                            "function": "sum"
                        }
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "project_relation".to_string(),
                    "bad_rollup".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await;
        assert_eq!(
            invalid.err().unwrap(),
            "rollup.function is not compatible with the target property"
        );
    });
}

#[test]
fn database_formulas_compute_without_persisting_stale_values() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_database(
            &pool,
            DATABASE_A,
            DATA_SOURCE_A,
            DATABASE_VIEW_A,
            "Tasks",
            BLOCK_A,
        )
        .await;
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "type": "number",
                        "number": {
                            "format": "number"
                        }
                    },
                    "Done": {
                        "id": "done",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    },
                    "Score": {
                        "id": "score_formula",
                        "name": "Score",
                        "type": "formula",
                        "formula": {
                            "expression": "prop(\"Estimate\") * 2"
                        }
                    },
                    "Score label": {
                        "id": "score_label_formula",
                        "name": "Score label",
                        "type": "formula",
                        "formula": {
                            "expression": "\"Score: \" + prop(\"Score\")"
                        }
                    },
                    "State": {
                        "id": "state_formula",
                        "name": "State",
                        "type": "formula",
                        "formula": {
                            "expression": "if(prop(\"Done\"), \"Complete\", \"Open\")"
                        }
                    },
                    "Broken": {
                        "id": "broken_formula",
                        "name": "Broken",
                        "type": "formula",
                        "formula": {
                            "expression": "prop(\"Estimate\") / 0"
                        }
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "estimate".to_string(),
                    "done".to_string(),
                    "score_formula".to_string(),
                    "score_label_formula".to_string(),
                    "state_formula".to_string(),
                    "broken_formula".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Write formula tests".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(4),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "done".to_string(),
                value: json!(true),
            },
        )
        .await
        .unwrap();

        let table = data_source_table::get_data_source_table_view(&pool, DATA_SOURCE_A, None, None)
            .await
            .unwrap();
        let table_json = serde_json::to_value(table).unwrap();
        let properties = &table_json["rows"][0]["properties"];
        assert_eq!(properties["Score"]["formula"]["type"], "number");
        assert_eq!(properties["Score"]["formula"]["number"].as_f64(), Some(8.0));
        assert_eq!(properties["Score label"]["formula"]["string"], "Score: 8");
        assert_eq!(properties["State"]["formula"]["string"], "Complete");
        assert_eq!(
            properties["Broken"]["formula"]["ganbaru_error"],
            "Broken: division by zero"
        );

        let stored_properties: String =
            sqlx::query_scalar("SELECT properties FROM notes_pages WHERE id = ?")
                .bind(PAGE_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        let stored_json: serde_json::Value = serde_json::from_str(&stored_properties).unwrap();
        assert!(stored_json.get("Score").is_none());
        assert!(stored_json.get("Score label").is_none());
        assert!(stored_json.get("State").is_none());
        assert!(stored_json.get("Broken").is_none());

        let invalid_edit = data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "score_formula".to_string(),
                value: json!(9),
            },
        )
        .await;
        assert_eq!(
            invalid_edit.err().unwrap(),
            "this property is read-only in the table view"
        );
    });
}

#[test]
fn database_formulas_reject_unknown_dependencies_cycles_and_dynamic_prop_calls() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_database(
            &pool,
            DATABASE_A,
            DATA_SOURCE_A,
            DATABASE_VIEW_A,
            "Tasks",
            BLOCK_A,
        )
        .await;

        let unknown_dependency = data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Bad": {
                        "id": "bad_formula",
                        "name": "Bad",
                        "type": "formula",
                        "formula": {
                            "expression": "prop(\"Missing\")"
                        }
                    }
                }),
                property_order: vec!["title".to_string(), "bad_formula".to_string()],
                hidden_property_ids: vec![],
            },
        )
        .await;
        assert_eq!(
            unknown_dependency.err().unwrap(),
            "formula references an unknown property"
        );

        let cycle = data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Alpha": {
                        "id": "alpha_formula",
                        "name": "Alpha",
                        "type": "formula",
                        "formula": {
                            "expression": "prop(\"Beta\") + 1"
                        }
                    },
                    "Beta": {
                        "id": "beta_formula",
                        "name": "Beta",
                        "type": "formula",
                        "formula": {
                            "expression": "prop(\"Alpha\") + 1"
                        }
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "alpha_formula".to_string(),
                    "beta_formula".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await;
        assert_eq!(
            cycle.err().unwrap(),
            "formula dependency cycle is not allowed"
        );

        let dynamic_prop = data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Bad": {
                        "id": "bad_formula",
                        "name": "Bad",
                        "type": "formula",
                        "formula": {
                            "expression": "prop(format(\"Name\"))"
                        }
                    }
                }),
                property_order: vec!["title".to_string(), "bad_formula".to_string()],
                hidden_property_ids: vec![],
            },
        )
        .await;
        assert_eq!(
            dynamic_prop.err().unwrap(),
            "prop() requires one literal property name"
        );
    });
}

#[test]
fn database_templates_create_row_pages_with_properties_and_body_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_database(
            &pool,
            DATABASE_A,
            DATA_SOURCE_A,
            DATABASE_VIEW_A,
            "Tasks",
            BLOCK_A,
        )
        .await;
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Priority": {
                        "id": "priority",
                        "name": "Priority",
                        "type": "select",
                        "select": {
                            "options": [
                                { "id": "high", "name": "High", "color": "red" }
                            ]
                        }
                    },
                    "Done": {
                        "id": "done",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "priority".to_string(),
                    "done".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Template source".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "priority".to_string(),
                value: json!("high"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "done".to_string(),
                value: json!(true),
            },
        )
        .await
        .unwrap();
        writes::update_block(
            &pool,
            BLOCK_B,
            block_update("paragraph", paragraph_payload("Plan first step")),
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_B),
                after: Some(BLOCK_B.to_string()),
                children: vec![block(
                    BLOCK_D,
                    "paragraph",
                    paragraph_payload("Review risks"),
                )],
            },
        )
        .await
        .unwrap();

        let template = data_source_templates::create_data_source_template_from_row(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceTemplateCreateFromRow {
                id: TEMPLATE_A.to_string(),
                source_page_id: PAGE_B.to_string(),
                name: "QA task".to_string(),
                is_default: Some(true),
            },
        )
        .await
        .unwrap();
        let template_json = serde_json::to_value(template).unwrap();
        assert_eq!(template_json["name"], "QA task");
        assert_eq!(template_json["block_count"], 2);
        assert_eq!(template_json["is_default"], true);

        let applied = data_source_templates::apply_data_source_template(
            &pool,
            DATA_SOURCE_A,
            TEMPLATE_A,
            NoteDataSourceTemplateApply {
                title: Some("QA pass".to_string()),
            },
        )
        .await
        .unwrap();
        let applied_json = serde_json::to_value(applied).unwrap();
        assert_eq!(
            applied_json["page"]["parent"]["data_source_id"],
            DATA_SOURCE_A
        );
        assert_eq!(
            applied_json["page"]["properties"]["Name"]["title"][0]["plain_text"],
            "QA pass"
        );
        assert_eq!(
            applied_json["page"]["properties"]["Priority"]["select"]["name"],
            "High"
        );
        assert_eq!(applied_json["page"]["properties"]["Done"]["checkbox"], true);
        assert_eq!(
            applied_json["blocks"]["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Plan first step"
        );
        assert_eq!(
            applied_json["blocks"]["results"][1]["paragraph"]["rich_text"][0]["plain_text"],
            "Review risks"
        );
    });
}

#[test]
fn database_buttons_require_confirmation_and_update_current_row_properties() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_database(
            &pool,
            DATABASE_A,
            DATA_SOURCE_A,
            DATABASE_VIEW_A,
            "Tasks",
            BLOCK_A,
        )
        .await;
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Done": {
                        "id": "done",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    },
                    "Finish": {
                        "id": "finish_button",
                        "name": "Finish",
                        "type": "button",
                        "button": {
                            "label": "Mark done",
                            "requires_confirmation": true,
                            "actions": [{
                                "type": "update_current_row_property",
                                "property_id": "done",
                                "property_name": "Done",
                                "property_type": "checkbox",
                                "value": true
                            }]
                        }
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "done".to_string(),
                    "finish_button".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Write button tests".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();

        let stored_properties: String =
            sqlx::query_scalar("SELECT properties FROM notes_pages WHERE id = ?")
                .bind(PAGE_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        let stored_json: serde_json::Value = serde_json::from_str(&stored_properties).unwrap();
        assert!(stored_json.get("Finish").is_none());

        let table = data_source_table::get_data_source_table_view(&pool, DATA_SOURCE_A, None, None)
            .await
            .unwrap();
        let table_json = serde_json::to_value(table).unwrap();
        assert_eq!(
            table_json["rows"][0]["properties"]["Finish"]["button"]["label"],
            "Mark done"
        );

        let unconfirmed = data_source_buttons::click_data_source_button(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceButtonClick {
                property_id: "finish_button".to_string(),
                confirmed: None,
            },
        )
        .await;
        assert_eq!(
            unconfirmed.err().unwrap(),
            "button action requires confirmation"
        );

        let updated = data_source_buttons::click_data_source_button(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceButtonClick {
                property_id: "finish_button".to_string(),
                confirmed: Some(true),
            },
        )
        .await
        .unwrap();
        let updated_json = serde_json::to_value(updated).unwrap();
        assert_eq!(updated_json["properties"]["Done"]["checkbox"], true);

        let invalid_broad_action = data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Finish": {
                        "id": "finish_button",
                        "name": "Finish",
                        "type": "button",
                        "button": {
                            "label": "Broad change",
                            "requires_confirmation": false,
                            "actions": [{
                                "type": "delete_pages"
                            }]
                        }
                    }
                }),
                property_order: vec!["title".to_string(), "finish_button".to_string()],
                hidden_property_ids: vec![],
            },
        )
        .await;
        assert_eq!(
            invalid_broad_action.err().unwrap(),
            "broad or destructive button actions require confirmation"
        );
    });
}

#[test]
fn board_database_view_groups_filters_sorts_and_moves_rows() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Status": {
                        "id": "status",
                        "name": "Status",
                        "type": "status",
                        "status": {
                            "options": [
                                { "id": "todo", "name": "To-do", "color": "gray", "group": "To-do" },
                                { "id": "doing", "name": "Doing", "color": "blue", "group": "In progress" },
                                { "id": "done", "name": "Done", "color": "green", "group": "Complete" }
                            ]
                        }
                    },
                    "Priority": {
                        "id": "priority",
                        "name": "Priority",
                        "type": "select",
                        "select": {
                            "options": [
                                { "id": "low", "name": "Low", "color": "blue" },
                                { "id": "high", "name": "High", "color": "red" }
                            ]
                        }
                    },
                    "Done": {
                        "id": "done_checkbox",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    },
                    "Due": {
                        "id": "due",
                        "name": "Due",
                        "type": "date",
                        "date": {}
                    },
                    "Owner": {
                        "id": "owner",
                        "name": "Owner",
                        "type": "people",
                        "people": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "status".to_string(),
                    "priority".to_string(),
                    "done_checkbox".to_string(),
                    "due".to_string(),
                    "owner".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();

        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Beta".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Alpha".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();

        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "status".to_string(),
                value: json!("Doing"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "priority".to_string(),
                value: json!("High"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "priority".to_string(),
                value: json!("Low"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "done_checkbox".to_string(),
                value: json!(true),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "due".to_string(),
                value: json!("2026-07-02"),
            },
        )
        .await
        .unwrap();

        let default_board =
            data_source_board::get_data_source_board_view(&pool, DATA_SOURCE_A, None, None)
                .await
                .unwrap();
        let default_json = serde_json::to_value(default_board).unwrap();
        assert_eq!(default_json["view"]["type"], "board");
        assert_eq!(
            default_json["view"]["configuration"]["board"]["group_property_id"],
            "status"
        );
        assert!(default_json["groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|group| group["id"] == "todo" && group["rows"].as_array().unwrap().is_empty()));
        assert!(default_json["groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|group| group["id"] == "doing" && group["rows"][0]["id"] == PAGE_B));
        assert!(default_json["groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|group| group["id"] == "__empty__" && group["rows"][0]["id"] == PAGE_C));

        let priority_board = data_source_board::update_data_source_board_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceBoardViewUpdate {
                filter: vec![NoteDataSourceTableFilter {
                    property_id: "title".to_string(),
                    condition: "contains".to_string(),
                    value: Some(json!("a")),
                }],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "title".to_string(),
                    direction: "ascending".to_string(),
                }],
                configuration: NoteDataSourceBoardConfigurationUpdate {
                    group_property_id: Some("priority".to_string()),
                    group_order: vec![
                        "high".to_string(),
                        "low".to_string(),
                        "__empty__".to_string(),
                    ],
                    hidden_group_ids: vec!["__empty__".to_string()],
                    visible_property_ids: vec![
                        "status".to_string(),
                        "done_checkbox".to_string(),
                        "due".to_string(),
                    ],
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let priority_json = serde_json::to_value(priority_board).unwrap();
        assert_eq!(
            priority_json["view"]["configuration"]["board"]["row_open_mode"],
            "side_panel"
        );
        assert_eq!(
            priority_json["view"]["configuration"]["board"]["visible_property_ids"],
            json!(["status", "done_checkbox", "due"])
        );
        assert!(priority_json["groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|group| group["id"] == "low" && group["rows"][0]["id"] == PAGE_C));
        assert!(priority_json["groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|group| group["id"] == "__empty__" && group["hidden"] == true));

        let moved = data_source_board::move_data_source_board_row(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceBoardRowMove {
                page_id: PAGE_C.to_string(),
                group_id: "high".to_string(),
            },
        )
        .await
        .unwrap();
        let moved_json = serde_json::to_value(moved).unwrap();
        let high_group = moved_json["groups"]
            .as_array()
            .unwrap()
            .iter()
            .find(|group| group["id"] == "high")
            .unwrap();
        assert_eq!(high_group["rows"].as_array().unwrap().len(), 2);
        assert!(high_group["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == PAGE_C
                && row["properties"]["Priority"]["select"]["name"] == "High"));
        assert!(moved_json["groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|group| group["id"] == "low" && group["rows"].as_array().unwrap().is_empty()));
    });
}

#[test]
fn gallery_database_view_persists_card_preview_filters_and_sorts() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Media tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Cover": {
                        "id": "cover_files",
                        "name": "Cover",
                        "type": "files",
                        "files": {}
                    },
                    "Status": {
                        "id": "status",
                        "name": "Status",
                        "type": "status",
                        "status": {
                            "options": [
                                { "id": "todo", "name": "To-do", "color": "gray", "group": "To-do" },
                                { "id": "doing", "name": "Doing", "color": "blue", "group": "In progress" }
                            ]
                        }
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "type": "number",
                        "number": { "format": "number" }
                    },
                    "Published": {
                        "id": "published",
                        "name": "Published",
                        "type": "checkbox",
                        "checkbox": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "cover_files".to_string(),
                    "status".to_string(),
                    "estimate".to_string(),
                    "published".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();

        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Alpha".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: Some(json!({
                    "Cover": {
                        "id": "cover_files",
                        "type": "files",
                        "files": [
                            {
                                "type": "external",
                                "external": { "url": "https://example.com/alpha.png" }
                            }
                        ]
                    }
                })),
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Beta".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(2),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(5),
            },
        )
        .await
        .unwrap();

        let default_gallery =
            data_source_gallery::get_data_source_gallery_view(&pool, DATA_SOURCE_A, None, None)
                .await
                .unwrap();
        let default_json = serde_json::to_value(default_gallery).unwrap();
        assert_eq!(default_json["view"]["type"], "gallery");
        assert_eq!(
            default_json["view"]["configuration"]["gallery"]["cover_source"],
            "page_cover"
        );
        let mut default_visible_property_ids = default_json["view"]["configuration"]["gallery"]
            ["visible_property_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        default_visible_property_ids.sort_unstable();
        assert_eq!(
            default_visible_property_ids,
            vec!["cover_files", "estimate", "published", "status"]
        );
        assert_eq!(default_json["rows"].as_array().unwrap().len(), 2);

        let invalid_cover = data_source_gallery::update_data_source_gallery_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceGalleryViewUpdate {
                filter: vec![],
                sorts: vec![],
                configuration: NoteDataSourceGalleryConfigurationUpdate {
                    cover_source: "files_property".to_string(),
                    cover_property_id: Some("estimate".to_string()),
                    visible_property_ids: vec![],
                    card_size: "medium".to_string(),
                    fit_image: false,
                    row_open_mode: "full_page".to_string(),
                },
            },
        )
        .await;
        match invalid_cover {
            Ok(_) => panic!("gallery accepted a non-files cover property"),
            Err(error) => assert!(error.contains("files property")),
        }

        let updated = data_source_gallery::update_data_source_gallery_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceGalleryViewUpdate {
                filter: vec![NoteDataSourceTableFilter {
                    property_id: "title".to_string(),
                    condition: "contains".to_string(),
                    value: Some(json!("a")),
                }],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "estimate".to_string(),
                    direction: "descending".to_string(),
                }],
                configuration: NoteDataSourceGalleryConfigurationUpdate {
                    cover_source: "files_property".to_string(),
                    cover_property_id: Some("cover_files".to_string()),
                    visible_property_ids: vec![
                        "estimate".to_string(),
                        "status".to_string(),
                        "estimate".to_string(),
                        "title".to_string(),
                        "missing".to_string(),
                    ],
                    card_size: "large".to_string(),
                    fit_image: true,
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let updated_json = serde_json::to_value(updated).unwrap();
        assert_eq!(
            updated_json["view"]["configuration"]["gallery"]["cover_property_id"],
            "cover_files"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["gallery"]["visible_property_ids"],
            json!(["estimate", "status"])
        );
        assert_eq!(
            updated_json["view"]["configuration"]["gallery"]["card_size"],
            "large"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["gallery"]["row_open_mode"],
            "side_panel"
        );
        assert_eq!(updated_json["rows"][0]["id"], PAGE_C);
        assert_eq!(updated_json["rows"][1]["id"], PAGE_B);
        let alpha = updated_json["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == PAGE_B)
            .unwrap();
        assert_eq!(
            alpha["properties"]["Cover"]["files"][0]["external"]["url"],
            "https://example.com/alpha.png"
        );
    });
}

#[test]
fn list_database_view_persists_visible_properties_grouping_filters_and_sorts() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "List tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Status": {
                        "id": "status",
                        "name": "Status",
                        "type": "status",
                        "status": {
                            "options": [
                                { "id": "todo", "name": "To-do", "color": "gray", "group": "To-do" },
                                { "id": "doing", "name": "Doing", "color": "blue", "group": "In progress" }
                            ]
                        }
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "type": "number",
                        "number": { "format": "number" }
                    },
                    "Published": {
                        "id": "published",
                        "name": "Published",
                        "type": "checkbox",
                        "checkbox": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "status".to_string(),
                    "estimate".to_string(),
                    "published".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();

        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Alpha".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Beta".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "status".to_string(),
                value: json!("Doing"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(2),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "status".to_string(),
                value: json!("To-do"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(5),
            },
        )
        .await
        .unwrap();

        let default_list =
            data_source_list::get_data_source_list_view(&pool, DATA_SOURCE_A, None, None)
                .await
                .unwrap();
        let default_json = serde_json::to_value(default_list).unwrap();
        assert_eq!(default_json["view"]["type"], "list");
        assert_eq!(
            default_json["view"]["configuration"]["list"]["group_property_id"],
            serde_json::Value::Null
        );
        assert_eq!(
            default_json["view"]["configuration"]["list"]["row_open_mode"],
            "side_panel"
        );
        assert_eq!(default_json["rows"].as_array().unwrap().len(), 2);

        let invalid_group = data_source_list::update_data_source_list_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceListViewUpdate {
                filter: vec![],
                sorts: vec![],
                configuration: NoteDataSourceListConfigurationUpdate {
                    group_property_id: Some("estimate".to_string()),
                    group_order: vec![],
                    hidden_group_ids: vec![],
                    visible_property_ids: vec![],
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await;
        match invalid_group {
            Ok(_) => panic!("list accepted a non-groupable property"),
            Err(error) => assert!(error.contains("group property type")),
        }

        let updated = data_source_list::update_data_source_list_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceListViewUpdate {
                filter: vec![NoteDataSourceTableFilter {
                    property_id: "title".to_string(),
                    condition: "contains".to_string(),
                    value: Some(json!("a")),
                }],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "estimate".to_string(),
                    direction: "descending".to_string(),
                }],
                configuration: NoteDataSourceListConfigurationUpdate {
                    group_property_id: Some("status".to_string()),
                    group_order: vec!["todo".to_string(), "doing".to_string()],
                    hidden_group_ids: vec!["todo".to_string()],
                    visible_property_ids: vec![
                        "estimate".to_string(),
                        "status".to_string(),
                        "title".to_string(),
                        "estimate".to_string(),
                        "missing".to_string(),
                    ],
                    row_open_mode: "full_page".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let updated_json = serde_json::to_value(updated).unwrap();
        assert_eq!(
            updated_json["view"]["configuration"]["list"]["group_property_id"],
            "status"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["list"]["hidden_group_ids"],
            json!(["todo"])
        );
        assert_eq!(
            updated_json["view"]["configuration"]["list"]["visible_property_ids"],
            json!(["estimate"])
        );
        assert_eq!(
            updated_json["view"]["configuration"]["list"]["row_open_mode"],
            "full_page"
        );
        assert_eq!(updated_json["rows"][0]["id"], PAGE_C);
        assert_eq!(updated_json["rows"][1]["id"], PAGE_B);
    });
}

#[test]
fn calendar_database_view_uses_date_ranges_filters_sorts_and_configuration() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Due": {
                        "id": "due",
                        "name": "Due",
                        "type": "date",
                        "date": {}
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "type": "number",
                        "number": { "format": "number" }
                    },
                    "Status": {
                        "id": "status",
                        "name": "Status",
                        "type": "status",
                        "status": {
                            "options": [
                                { "id": "todo", "name": "To-do", "color": "gray" },
                                { "id": "doing", "name": "Doing", "color": "blue" }
                            ]
                        }
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "due".to_string(),
                    "estimate".to_string(),
                    "status".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();

        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Alpha".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Beta".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "due".to_string(),
                value: json!("2026-07-10"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(2),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "due".to_string(),
                value: json!({
                    "start": "2026-07-30",
                    "end": "2026-08-02",
                    "time_zone": null
                }),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(5),
            },
        )
        .await
        .unwrap();

        let default_calendar =
            data_source_calendar::get_data_source_calendar_view(&pool, DATA_SOURCE_A, None, None)
                .await
                .unwrap();
        let default_json = serde_json::to_value(default_calendar).unwrap();
        assert_eq!(default_json["view"]["type"], "calendar");
        assert_eq!(
            default_json["view"]["configuration"]["calendar"]["date_property_id"],
            "due"
        );
        assert_eq!(
            default_json["view"]["configuration"]["calendar"]["row_open_mode"],
            "side_panel"
        );

        let invalid_date_property = data_source_calendar::update_data_source_calendar_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceCalendarViewUpdate {
                filter: vec![],
                sorts: vec![],
                configuration: NoteDataSourceCalendarConfigurationUpdate {
                    date_property_id: Some("estimate".to_string()),
                    range_start: "2026-07-01".to_string(),
                    range_end: "2026-07-31".to_string(),
                    visible_property_ids: vec![],
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await;
        match invalid_date_property {
            Ok(_) => panic!("calendar accepted a non-date property"),
            Err(error) => assert!(error.contains("date property")),
        }

        let updated = data_source_calendar::update_data_source_calendar_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceCalendarViewUpdate {
                filter: vec![NoteDataSourceTableFilter {
                    property_id: "title".to_string(),
                    condition: "contains".to_string(),
                    value: Some(json!("a")),
                }],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "estimate".to_string(),
                    direction: "descending".to_string(),
                }],
                configuration: NoteDataSourceCalendarConfigurationUpdate {
                    date_property_id: Some("due".to_string()),
                    range_start: "2026-07-01".to_string(),
                    range_end: "2026-07-31".to_string(),
                    visible_property_ids: vec![
                        "estimate".to_string(),
                        "due".to_string(),
                        "title".to_string(),
                        "estimate".to_string(),
                        "missing".to_string(),
                    ],
                    row_open_mode: "full_page".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let updated_json = serde_json::to_value(updated).unwrap();
        assert_eq!(
            updated_json["view"]["configuration"]["calendar"]["range_start"],
            "2026-07-01"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["calendar"]["range_end"],
            "2026-07-31"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["calendar"]["visible_property_ids"],
            json!(["estimate"])
        );
        assert_eq!(
            updated_json["view"]["configuration"]["calendar"]["row_open_mode"],
            "full_page"
        );
        assert_eq!(updated_json["rows"][0]["id"], PAGE_C);
        assert_eq!(updated_json["rows"][1]["id"], PAGE_B);

        let august = data_source_calendar::update_data_source_calendar_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceCalendarViewUpdate {
                filter: vec![],
                sorts: vec![],
                configuration: NoteDataSourceCalendarConfigurationUpdate {
                    date_property_id: Some("due".to_string()),
                    range_start: "2026-08-01".to_string(),
                    range_end: "2026-08-31".to_string(),
                    visible_property_ids: vec![],
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let august_json = serde_json::to_value(august).unwrap();
        assert_eq!(august_json["rows"].as_array().unwrap().len(), 1);
        assert_eq!(august_json["rows"][0]["id"], PAGE_C);
    });
}

#[test]
fn timeline_database_view_uses_date_ranges_grouping_filters_sorts_and_configuration() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Window": {
                        "id": "window",
                        "name": "Window",
                        "type": "date",
                        "date": {}
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "type": "number",
                        "number": { "format": "number" }
                    },
                    "Status": {
                        "id": "status",
                        "name": "Status",
                        "type": "status",
                        "status": {
                            "options": [
                                { "id": "todo", "name": "To-do", "color": "gray" },
                                { "id": "doing", "name": "Doing", "color": "blue" }
                            ]
                        }
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "window".to_string(),
                    "estimate".to_string(),
                    "status".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();

        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Alpha".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Beta".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "window".to_string(),
                value: json!({
                    "start": "2026-07-10",
                    "end": "2026-07-15",
                    "time_zone": null
                }),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(2),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "window".to_string(),
                value: json!({
                    "start": "2026-07-30",
                    "end": "2026-08-02",
                    "time_zone": null
                }),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(5),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "status".to_string(),
                value: json!("Doing"),
            },
        )
        .await
        .unwrap();

        let default_timeline =
            data_source_timeline::get_data_source_timeline_view(&pool, DATA_SOURCE_A, None, None)
                .await
                .unwrap();
        let default_json = serde_json::to_value(default_timeline).unwrap();
        assert_eq!(default_json["view"]["type"], "timeline");
        assert_eq!(
            default_json["view"]["configuration"]["timeline"]["date_property_id"],
            "window"
        );
        assert_eq!(
            default_json["view"]["configuration"]["timeline"]["group_property_id"],
            serde_json::Value::Null
        );

        let invalid_group = data_source_timeline::update_data_source_timeline_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceTimelineViewUpdate {
                filter: vec![],
                sorts: vec![],
                configuration: NoteDataSourceTimelineConfigurationUpdate {
                    date_property_id: Some("window".to_string()),
                    group_property_id: Some("estimate".to_string()),
                    group_order: vec![],
                    hidden_group_ids: vec![],
                    range_start: "2026-07-01".to_string(),
                    range_end: "2026-07-31".to_string(),
                    visible_property_ids: vec![],
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await;
        match invalid_group {
            Ok(_) => panic!("timeline accepted a non-groupable property"),
            Err(error) => assert!(error.contains("group property type")),
        }

        let updated = data_source_timeline::update_data_source_timeline_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceTimelineViewUpdate {
                filter: vec![NoteDataSourceTableFilter {
                    property_id: "title".to_string(),
                    condition: "contains".to_string(),
                    value: Some(json!("a")),
                }],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "estimate".to_string(),
                    direction: "descending".to_string(),
                }],
                configuration: NoteDataSourceTimelineConfigurationUpdate {
                    date_property_id: Some("window".to_string()),
                    group_property_id: Some("status".to_string()),
                    group_order: vec!["doing".to_string(), "todo".to_string()],
                    hidden_group_ids: vec!["todo".to_string()],
                    range_start: "2026-07-01".to_string(),
                    range_end: "2026-07-31".to_string(),
                    visible_property_ids: vec![
                        "estimate".to_string(),
                        "window".to_string(),
                        "status".to_string(),
                        "title".to_string(),
                        "estimate".to_string(),
                    ],
                    row_open_mode: "full_page".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let updated_json = serde_json::to_value(updated).unwrap();
        assert_eq!(
            updated_json["view"]["configuration"]["timeline"]["group_property_id"],
            "status"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["timeline"]["hidden_group_ids"],
            json!(["todo"])
        );
        assert_eq!(
            updated_json["view"]["configuration"]["timeline"]["visible_property_ids"],
            json!(["estimate"])
        );
        assert_eq!(
            updated_json["view"]["configuration"]["timeline"]["row_open_mode"],
            "full_page"
        );
        assert_eq!(updated_json["rows"][0]["id"], PAGE_C);
        assert_eq!(updated_json["rows"][1]["id"], PAGE_B);

        let august = data_source_timeline::update_data_source_timeline_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceTimelineViewUpdate {
                filter: vec![],
                sorts: vec![],
                configuration: NoteDataSourceTimelineConfigurationUpdate {
                    date_property_id: Some("window".to_string()),
                    group_property_id: None,
                    group_order: vec![],
                    hidden_group_ids: vec![],
                    range_start: "2026-08-01".to_string(),
                    range_end: "2026-08-31".to_string(),
                    visible_property_ids: vec![],
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let august_json = serde_json::to_value(august).unwrap();
        assert_eq!(august_json["rows"].as_array().unwrap().len(), 1);
        assert_eq!(august_json["rows"][0]["id"], PAGE_C);
    });
}

#[test]
fn update_local_data_source_schema() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();

        let updated = data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "description": "Row title",
                        "type": "title",
                        "title": {}
                    },
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "description": "",
                        "type": "rich_text",
                        "rich_text": {}
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "description": "",
                        "type": "number",
                        "number": { "format": "percent" }
                    },
                    "Priority": {
                        "id": "priority",
                        "name": "Priority",
                        "description": "",
                        "type": "select",
                        "select": {
                            "options": [
                                { "id": "low", "name": "Low", "color": "blue" },
                                { "id": "high", "name": "High", "color": "red" }
                            ]
                        }
                    },
                    "Tags": {
                        "id": "tags",
                        "name": "Tags",
                        "description": "",
                        "type": "multi_select",
                        "multi_select": {
                            "options": [
                                { "id": "home", "name": "Home", "color": "green" }
                            ]
                        }
                    },
                    "Status": {
                        "id": "status",
                        "name": "Status",
                        "description": "",
                        "type": "status",
                        "status": {
                            "options": [
                                {
                                    "id": "todo",
                                    "name": "Todo",
                                    "color": "default",
                                    "group": "To-do"
                                },
                                {
                                    "id": "doing",
                                    "name": "Doing",
                                    "color": "blue",
                                    "group": "In progress"
                                },
                                {
                                    "id": "done",
                                    "name": "Done",
                                    "color": "green",
                                    "group": "Complete"
                                }
                            ]
                        }
                    },
                    "Due": { "id": "due", "name": "Due", "type": "date", "date": {} },
                    "Done": {
                        "id": "done_checkbox",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    },
                    "URL": { "id": "url", "name": "URL", "type": "url", "url": {} },
                    "Email": { "id": "email", "name": "Email", "type": "email", "email": {} },
                    "Phone": {
                        "id": "phone",
                        "name": "Phone",
                        "type": "phone_number",
                        "phone_number": {}
                    },
                    "Files": { "id": "files", "name": "Files", "type": "files", "files": {} },
                    "People": {
                        "id": "people",
                        "name": "People",
                        "type": "people",
                        "people": {}
                    },
                    "Created": {
                        "id": "created",
                        "name": "Created",
                        "type": "created_time",
                        "created_time": {}
                    },
                    "Created by": {
                        "id": "created_by",
                        "name": "Created by",
                        "type": "created_by",
                        "created_by": {}
                    },
                    "Edited": {
                        "id": "edited",
                        "name": "Edited",
                        "type": "last_edited_time",
                        "last_edited_time": {}
                    },
                    "Edited by": {
                        "id": "edited_by",
                        "name": "Edited by",
                        "type": "last_edited_by",
                        "last_edited_by": {}
                    },
                    "Task ID": {
                        "id": "task_id",
                        "name": "Task ID",
                        "type": "unique_id",
                        "unique_id": { "prefix": "TASK" }
                    },
                    "Place": { "id": "place", "name": "Place", "type": "place", "place": {} }
                }),
                property_order: vec![
                    "title".to_string(),
                    "status".to_string(),
                    "priority".to_string(),
                    "details".to_string(),
                    "estimate".to_string(),
                    "tags".to_string(),
                    "due".to_string(),
                    "done_checkbox".to_string(),
                    "url".to_string(),
                    "email".to_string(),
                    "phone".to_string(),
                    "files".to_string(),
                    "people".to_string(),
                    "created".to_string(),
                    "created_by".to_string(),
                    "edited".to_string(),
                    "edited_by".to_string(),
                    "task_id".to_string(),
                    "place".to_string(),
                ],
                hidden_property_ids: vec!["details".to_string(), "files".to_string()],
            },
        )
        .await
        .unwrap();

        let updated_json = serde_json::to_value(updated).unwrap();
        assert_eq!(
            updated_json["data_source"]["properties"]["Priority"]["select"]["options"][1]["name"],
            "High"
        );
        assert_eq!(
            updated_json["data_source"]["properties"]["Estimate"]["number"]["format"],
            "percent"
        );
        assert_eq!(
            updated_json["data_source"]["properties"]["Task ID"]["unique_id"]["prefix"],
            "TASK"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["table"]["property_order"][1],
            "status"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["table"]["hidden_property_ids"][0],
            "details"
        );

        let stored_properties: String =
            sqlx::query_scalar("SELECT properties FROM notes_data_sources WHERE id = ?")
                .bind(DATA_SOURCE_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        let stored_properties_json: serde_json::Value =
            serde_json::from_str(&stored_properties).unwrap();
        assert_eq!(stored_properties_json["Place"]["type"], "place");

        let configuration: String =
            sqlx::query_scalar("SELECT configuration FROM notes_database_views WHERE id = ?")
                .bind(DATABASE_VIEW_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        let configuration_json: serde_json::Value = serde_json::from_str(&configuration).unwrap();
        assert_eq!(
            configuration_json["table"]["hidden_property_ids"],
            json!(["details", "files"])
        );
    });
}

#[test]
fn update_local_data_source_schema_rejects_title_removal() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();

        let result = data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "type": "rich_text",
                        "rich_text": {}
                    }
                }),
                property_order: vec!["details".to_string()],
                hidden_property_ids: vec![],
            },
        )
        .await;

        assert_eq!(
            result.err().unwrap(),
            "data source schema must contain exactly one title property"
        );
    });
}

#[test]
fn create_child_page_from_block_moves_nested_children_and_syncs_page_state() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Project"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    paragraph_payload("Nested detail"),
                )],
            },
        )
        .await
        .unwrap();

        writes::create_child_page_from_block(
            &pool,
            BLOCK_B,
            NoteChildPageFromBlockCreate {
                first_block_id: BLOCK_D.to_string(),
                title: None,
            },
        )
        .await
        .unwrap();

        let parent_block = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let parent_json = serde_json::to_value(parent_block).unwrap();
        assert_eq!(parent_json["type"], "child_page");
        assert_eq!(parent_json["child_page"]["title"], "Project");
        assert_eq!(parent_json["has_children"], false);

        let child_page = reads::get_page(&pool, BLOCK_B, false).await.unwrap();
        let child_page_json = serde_json::to_value(child_page).unwrap();
        assert_eq!(child_page_json["parent"]["page_id"], PAGE_A);

        let child_blocks = reads::get_block_children(&pool, BLOCK_B, None, Some(10))
            .await
            .unwrap();
        let child_blocks_json = serde_json::to_value(child_blocks).unwrap();
        assert_eq!(child_blocks_json["results"][0]["id"], BLOCK_C);
        assert_eq!(
            child_blocks_json["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Nested detail"
        );

        writes::update_page(
            &pool,
            BLOCK_B,
            super::models::NotePageUpdate {
                title: Some("Renamed child".to_string()),
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Unset,
                cover: OptionalJsonValue::Unset,
            },
        )
        .await
        .unwrap();
        let renamed_block = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let renamed_json = serde_json::to_value(renamed_block).unwrap();
        assert_eq!(renamed_json["child_page"]["title"], "Renamed child");

        writes::move_block(
            &pool,
            BLOCK_B,
            NoteMoveBlock {
                parent: block_parent(BLOCK_A),
                after: None,
                before: None,
            },
        )
        .await
        .unwrap();
        let moved_child_page = reads::get_page(&pool, BLOCK_B, false).await.unwrap();
        let moved_child_page_json = serde_json::to_value(moved_child_page).unwrap();
        assert_eq!(moved_child_page_json["parent"]["block_id"], BLOCK_A);

        writes::trash_page(&pool, BLOCK_B, true).await.unwrap();
        assert!(reads::get_block(&pool, BLOCK_B, false).await.is_err());
        writes::trash_page(&pool, BLOCK_B, false).await.unwrap();
        assert_eq!(
            serde_json::to_value(reads::get_block(&pool, BLOCK_B, false).await.unwrap()).unwrap()
                ["type"],
            "child_page"
        );
    });
}

#[test]
fn duplicate_block_rejects_child_page_blocks_until_page_duplication_exists() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested page".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: Some(BLOCK_A.to_string()),
            },
        )
        .await
        .unwrap();

        let result = writes::duplicate_block(
            &pool,
            PAGE_B,
            NoteDuplicateBlock {
                duplicated_block_ids: vec![NoteDuplicatedBlockId {
                    source_id: PAGE_B.to_string(),
                    duplicate_id: BLOCK_C.to_string(),
                }],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("child_page blocks must be duplicated through page duplication".to_string())
        );
    });
}

#[test]
fn duplicate_blocks_copies_loaded_subtrees_and_block_comments() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Nested"))],
            },
        )
        .await
        .unwrap();
        comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_A.to_string(),
                parent: Some(block_parent(BLOCK_C)),
                discussion_id: None,
                anchor: None,
                rich_text: vec![rich_text("Nested comment")],
            },
        )
        .await
        .unwrap();

        let duplicated = writes::duplicate_blocks(
            &pool,
            NoteDuplicateBlocks {
                block_ids: vec![BLOCK_B.to_string()],
                duplicated_block_ids: vec![
                    NoteDuplicatedBlockId {
                        source_id: BLOCK_B.to_string(),
                        duplicate_id: BLOCK_D.to_string(),
                    },
                    NoteDuplicatedBlockId {
                        source_id: BLOCK_C.to_string(),
                        duplicate_id: BLOCK_E.to_string(),
                    },
                ],
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_B.to_string()),
                before: None,
                include_trashed_sources: None,
            },
        )
        .await
        .unwrap();
        let duplicated_json = serde_json::to_value(duplicated).unwrap();
        assert_eq!(duplicated_json["results"][0]["id"], BLOCK_D);

        let duplicated_children = reads::get_block_children(&pool, BLOCK_D, None, Some(10))
            .await
            .unwrap();
        let duplicated_children_json = serde_json::to_value(duplicated_children).unwrap();
        assert_eq!(duplicated_children_json["results"][0]["id"], BLOCK_E);
        assert_eq!(
            duplicated_children_json["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Nested"
        );

        let threads = comments::list_comments(&pool, PAGE_A, false).await.unwrap();
        let threads_json = serde_json::to_value(threads).unwrap();
        assert!(threads_json.as_array().unwrap().iter().any(|thread| {
            thread["parent"]["block_id"] == BLOCK_E
                && thread["comments"][0]["rich_text"][0]["plain_text"] == "Nested comment"
        }));
    });
}

#[test]
fn move_blocks_moves_subtrees_updates_comment_pages_and_rejects_cycles() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Destination".to_string(),
                parent: workspace_parent(),
                first_block_id: BLOCK_F.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Nested"))],
            },
        )
        .await
        .unwrap();
        comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_A.to_string(),
                parent: Some(block_parent(BLOCK_C)),
                discussion_id: None,
                anchor: None,
                rich_text: vec![rich_text("Move with block")],
            },
        )
        .await
        .unwrap();

        let cycle = writes::move_blocks(
            &pool,
            NoteMoveBlocks {
                block_ids: vec![BLOCK_B.to_string()],
                parent: block_parent(BLOCK_C),
                after: None,
                before: None,
            },
        )
        .await;
        assert_eq!(
            cycle.err(),
            Some("block cannot be moved under its descendant".to_string())
        );

        writes::move_blocks(
            &pool,
            NoteMoveBlocks {
                block_ids: vec![BLOCK_B.to_string()],
                parent: page_parent(PAGE_B),
                after: Some(BLOCK_F.to_string()),
                before: None,
            },
        )
        .await
        .unwrap();

        let destination_children = reads::get_block_children(&pool, PAGE_B, None, Some(10))
            .await
            .unwrap();
        let destination_children_json = serde_json::to_value(destination_children).unwrap();
        assert_eq!(destination_children_json["results"][1]["id"], BLOCK_B);
        let moved_nested = reads::get_block(&pool, BLOCK_C, false).await.unwrap();
        let moved_nested_json = serde_json::to_value(moved_nested).unwrap();
        assert_eq!(moved_nested_json["parent"]["block_id"], BLOCK_B);

        let source_threads = comments::list_comments(&pool, PAGE_A, false).await.unwrap();
        assert!(serde_json::to_value(source_threads)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty());
        let destination_threads = comments::list_comments(&pool, PAGE_B, false).await.unwrap();
        let destination_threads_json = serde_json::to_value(destination_threads).unwrap();
        assert_eq!(destination_threads_json[0]["parent"]["block_id"], BLOCK_C);
        assert_eq!(
            destination_threads_json[0]["comments"][0]["rich_text"][0]["plain_text"],
            "Move with block"
        );
    });
}

#[test]
fn trash_blocks_trashes_selected_roots_and_loaded_descendants_once() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Nested"))],
            },
        )
        .await
        .unwrap();

        writes::trash_blocks(
            &pool,
            NoteTrashBlocks {
                block_ids: vec![BLOCK_B.to_string(), BLOCK_C.to_string()],
                in_trash: Some(true),
            },
        )
        .await
        .unwrap();

        assert!(reads::get_block(&pool, BLOCK_B, false).await.is_err());
        assert!(reads::get_block(&pool, BLOCK_C, false).await.is_err());
        let trashed_parent =
            serde_json::to_value(reads::get_block(&pool, BLOCK_B, true).await.unwrap()).unwrap();
        let trashed_child =
            serde_json::to_value(reads::get_block(&pool, BLOCK_C, true).await.unwrap()).unwrap();
        assert_eq!(trashed_parent["in_trash"], true);
        assert_eq!(trashed_child["in_trash"], true);
    });
}

#[test]
fn duplicate_page_copies_metadata_and_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: None,
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Value(json!({
                    "type": "emoji",
                    "emoji": "📌"
                })),
                cover: OptionalJsonValue::Value(json!({
                    "type": "external",
                    "external": {
                        "url": "https://example.com/cover.jpg"
                    }
                })),
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Detail"))],
            },
        )
        .await
        .unwrap();

        let duplicated = writes::duplicate_page(
            &pool,
            PAGE_A,
            NoteDuplicatePage {
                title: Some("Copy of First page".to_string()),
            },
        )
        .await
        .unwrap();
        let duplicated_json = serde_json::to_value(duplicated).unwrap();
        let duplicated_id = duplicated_json["page"]["id"].as_str().unwrap();
        assert_ne!(duplicated_id, PAGE_A);
        assert_eq!(duplicated_json["page"]["parent"]["type"], "workspace");
        assert_eq!(
            duplicated_json["page"]["properties"]["title"]["title"][0]["plain_text"],
            "Copy of First page"
        );
        assert_eq!(duplicated_json["page"]["icon"]["emoji"], "📌");
        assert_eq!(
            duplicated_json["page"]["cover"]["external"]["url"],
            "https://example.com/cover.jpg"
        );
        assert_eq!(
            duplicated_json["blocks"]["results"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            duplicated_json["blocks"]["results"][1]["paragraph"]["rich_text"][0]["plain_text"],
            "Detail"
        );
        assert_ne!(duplicated_json["blocks"]["results"][0]["id"], BLOCK_A);
        assert_ne!(duplicated_json["blocks"]["results"][1]["id"], BLOCK_B);
    });
}

#[test]
fn duplicate_page_copies_nested_child_pages() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: Some(BLOCK_A.to_string()),
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_C.to_string(),
                title: "Leaf".to_string(),
                parent: page_parent(PAGE_B),
                first_block_id: BLOCK_C.to_string(),
                after_block_id: Some(BLOCK_B.to_string()),
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_C),
                after: Some(BLOCK_C.to_string()),
                children: vec![block(
                    BLOCK_D,
                    "paragraph",
                    paragraph_payload("Leaf detail"),
                )],
            },
        )
        .await
        .unwrap();

        let duplicated = writes::duplicate_page(
            &pool,
            PAGE_B,
            NoteDuplicatePage {
                title: Some("Copy of Nested".to_string()),
            },
        )
        .await
        .unwrap();
        let duplicated_json = serde_json::to_value(duplicated).unwrap();
        let duplicated_id = duplicated_json["page"]["id"].as_str().unwrap();
        assert_eq!(duplicated_json["page"]["parent"]["page_id"], PAGE_A);
        assert_eq!(
            duplicated_json["page"]["properties"]["title"]["title"][0]["plain_text"],
            "Copy of Nested"
        );

        let parent_blocks = reads::get_block_children(&pool, PAGE_A, None, Some(10))
            .await
            .unwrap();
        let parent_blocks_json = serde_json::to_value(parent_blocks).unwrap();
        assert_eq!(parent_blocks_json["results"][1]["id"], PAGE_B);
        assert_eq!(parent_blocks_json["results"][2]["id"], duplicated_id);
        assert_eq!(
            parent_blocks_json["results"][2]["child_page"]["title"],
            "Copy of Nested"
        );

        let child_page_id: String =
            sqlx::query_scalar("SELECT id FROM notes_pages WHERE parent_page_id = ?")
                .bind(duplicated_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_ne!(child_page_id, PAGE_C);
        let duplicated_blocks = reads::get_block_children(&pool, duplicated_id, None, Some(10))
            .await
            .unwrap();
        let duplicated_blocks_json = serde_json::to_value(duplicated_blocks).unwrap();
        assert_eq!(duplicated_blocks_json["results"][1]["id"], child_page_id);
        assert_eq!(
            duplicated_blocks_json["results"][1]["child_page"]["title"],
            "Leaf"
        );

        let duplicated_child_blocks =
            reads::get_block_children(&pool, &child_page_id, None, Some(10))
                .await
                .unwrap();
        let duplicated_child_blocks_json = serde_json::to_value(duplicated_child_blocks).unwrap();
        assert_eq!(
            duplicated_child_blocks_json["results"][1]["paragraph"]["rich_text"][0]["plain_text"],
            "Leaf detail"
        );
    });
}

#[test]
fn page_rename_and_trash_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: Some("Renamed".to_string()),
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Unset,
                cover: OptionalJsonValue::Unset,
            },
        )
        .await
        .unwrap();
        let stored_title: String = sqlx::query_scalar("SELECT title FROM notes_pages WHERE id = ?")
            .bind(PAGE_A)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(stored_title, "Renamed");

        writes::trash_page(&pool, PAGE_A, true).await.unwrap();
        assert!(reads::list_pages(&pool).await.unwrap().is_empty());
        let trashed_pages = reads::list_trashed_pages(&pool).await.unwrap();
        assert_eq!(trashed_pages.len(), 1);

        writes::trash_page(&pool, PAGE_A, false).await.unwrap();
        assert_eq!(reads::list_pages(&pool).await.unwrap().len(), 1);
        assert!(reads::list_trashed_pages(&pool).await.unwrap().is_empty());
    });
}

#[test]
fn page_archive_and_unarchive_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::archive_page(&pool, PAGE_A, true).await.unwrap();
        assert!(reads::list_pages(&pool).await.unwrap().is_empty());
        assert!(reads::load_page(&pool, PAGE_A).await.is_err());
        let archived_pages = reads::list_archived_pages(&pool).await.unwrap();
        assert_eq!(archived_pages.len(), 1);
        let archived_page = serde_json::to_value(&archived_pages[0]).unwrap();
        assert_eq!(archived_page["archived"], true);
        assert_eq!(archived_page["in_trash"], false);

        let child_result = writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Blocked child".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await;
        let Err(child_error) = child_result else {
            panic!("archived parent unexpectedly accepted a child page");
        };
        assert_eq!(child_error, "parent page not found");

        writes::archive_page(&pool, PAGE_A, false).await.unwrap();
        assert_eq!(reads::list_pages(&pool).await.unwrap().len(), 1);
        assert!(reads::list_archived_pages(&pool).await.unwrap().is_empty());
        assert!(reads::load_page(&pool, PAGE_A).await.is_ok());
    });
}

#[test]
fn unarchiving_nested_page_with_archived_parent_promotes_to_workspace() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();

        writes::archive_page(&pool, PAGE_A, true).await.unwrap();
        writes::archive_page(&pool, PAGE_B, true).await.unwrap();
        let restored = writes::archive_page(&pool, PAGE_B, false).await.unwrap();
        let restored_json = serde_json::to_value(restored).unwrap();
        assert_eq!(restored_json["parent"]["type"], "workspace");
        assert!(reads::get_block(&pool, PAGE_B, false).await.is_err());
    });
}

#[test]
fn trashing_archived_page_clears_archive_state() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::archive_page(&pool, PAGE_A, true).await.unwrap();
        let trashed = writes::trash_page(&pool, PAGE_A, true).await.unwrap();
        let trashed_page = serde_json::to_value(trashed).unwrap();
        assert_eq!(trashed_page["in_trash"], true);
        assert_eq!(trashed_page["archived"], false);
        assert!(reads::list_archived_pages(&pool).await.unwrap().is_empty());
        assert_eq!(reads::list_trashed_pages(&pool).await.unwrap().len(), 1);
    });
}

#[test]
fn trashing_parent_page_updates_descendant_pages() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_C.to_string(),
                title: "Leaf".to_string(),
                parent: page_parent(PAGE_B),
                first_block_id: BLOCK_C.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();

        writes::trash_page(&pool, PAGE_A, true).await.unwrap();
        assert!(reads::list_pages(&pool).await.unwrap().is_empty());
        assert_eq!(reads::list_trashed_pages(&pool).await.unwrap().len(), 3);
        assert!(reads::get_block(&pool, PAGE_B, false).await.is_err());

        writes::trash_page(&pool, PAGE_A, false).await.unwrap();
        assert_eq!(reads::list_pages(&pool).await.unwrap().len(), 3);
        assert!(reads::list_trashed_pages(&pool).await.unwrap().is_empty());
        assert!(reads::get_block(&pool, PAGE_B, false).await.is_ok());
    });
}

#[test]
fn trashing_nested_page_hides_and_restores_child_page_block() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();

        writes::trash_page(&pool, PAGE_B, true).await.unwrap();
        assert!(reads::get_block(&pool, PAGE_B, false).await.is_err());
        let trashed_block = reads::get_block(&pool, PAGE_B, true).await.unwrap();
        let trashed_block_json = serde_json::to_value(trashed_block).unwrap();
        assert_eq!(trashed_block_json["in_trash"], true);

        writes::trash_page(&pool, PAGE_B, false).await.unwrap();
        assert!(reads::get_block(&pool, PAGE_B, false).await.is_ok());
    });
}

#[test]
fn restoring_nested_page_with_trashed_parent_promotes_to_workspace() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();

        writes::trash_page(&pool, PAGE_A, true).await.unwrap();
        let restored = writes::trash_page(&pool, PAGE_B, false).await.unwrap();
        let restored_json = serde_json::to_value(restored).unwrap();
        assert_eq!(restored_json["parent"]["type"], "workspace");
        assert_eq!(reads::list_pages(&pool).await.unwrap().len(), 1);
        assert_eq!(reads::list_trashed_pages(&pool).await.unwrap().len(), 1);
        assert!(reads::get_block(&pool, PAGE_B, false).await.is_err());
    });
}

#[test]
fn restoring_page_with_missing_parent_promotes_to_workspace() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::trash_page(&pool, PAGE_A, true).await.unwrap();
        sqlx::raw_sql("PRAGMA foreign_keys=OFF")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE notes_pages
             SET parent_type = 'page_id',
                 parent_page_id = ?,
                 parent_block_id = NULL
             WHERE id = ?",
        )
        .bind(PAGE_B)
        .bind(PAGE_A)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::raw_sql("PRAGMA foreign_keys=ON")
            .execute(&pool)
            .await
            .unwrap();

        let restored = writes::trash_page(&pool, PAGE_A, false).await.unwrap();
        let restored_json = serde_json::to_value(restored).unwrap();
        assert_eq!(restored_json["parent"]["type"], "workspace");
        assert!(reads::get_page(&pool, PAGE_A, false).await.is_ok());
    });
}

#[test]
fn permanent_page_delete_requires_trash() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let result = writes::permanently_delete_page(&pool, PAGE_A).await;
        assert_eq!(
            result,
            Err("notes page must be in trash before permanent delete".to_string())
        );
        assert_eq!(reads::list_pages(&pool).await.unwrap().len(), 1);
    });
}

#[test]
fn permanent_page_delete_removes_subtree_and_paired_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Leaf"))],
            },
        )
        .await
        .unwrap();
        writes::create_child_page_from_block(
            &pool,
            BLOCK_C,
            NoteChildPageFromBlockCreate {
                first_block_id: BLOCK_D.to_string(),
                title: None,
            },
        )
        .await
        .unwrap();

        writes::trash_page(&pool, PAGE_A, true).await.unwrap();
        let deleted_ids = writes::permanently_delete_page(&pool, PAGE_A)
            .await
            .unwrap();
        assert!(deleted_ids.contains(&PAGE_A.to_string()));
        assert!(deleted_ids.contains(&PAGE_B.to_string()));
        assert!(deleted_ids.contains(&BLOCK_C.to_string()));
        assert!(reads::list_pages(&pool).await.unwrap().is_empty());
        assert!(reads::list_trashed_pages(&pool).await.unwrap().is_empty());
        let page_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes_pages")
            .fetch_one(&pool)
            .await
            .unwrap();
        let block_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes_blocks")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(page_count, 0);
        assert_eq!(block_count, 0);
    });
}

#[test]
fn page_breadcrumb_resolves_unloaded_ancestor_rows() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_A.to_string(),
                title: "Root".to_string(),
                parent: workspace_parent(),
                first_block_id: BLOCK_A.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Child".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_C.to_string(),
                title: "Leaf".to_string(),
                parent: page_parent(PAGE_B),
                first_block_id: BLOCK_C.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();

        let breadcrumb = reads::get_page_breadcrumb(&pool, PAGE_C).await.unwrap();
        let breadcrumb_json = serde_json::to_value(breadcrumb).unwrap();
        assert_eq!(breadcrumb_json.as_array().unwrap().len(), 3);
        assert_eq!(breadcrumb_json[0]["id"], PAGE_A);
        assert_eq!(breadcrumb_json[0]["status"], "active");
        assert_eq!(breadcrumb_json[1]["id"], PAGE_B);
        assert_eq!(breadcrumb_json[1]["status"], "active");
        assert_eq!(breadcrumb_json[2]["id"], PAGE_C);
        assert_eq!(breadcrumb_json[2]["current"], true);
    });
}

#[test]
fn page_breadcrumb_marks_unavailable_ancestors() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_A.to_string(),
                title: "Archived root".to_string(),
                parent: workspace_parent(),
                first_block_id: BLOCK_A.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Trashed child".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_C.to_string(),
                title: "Leaf".to_string(),
                parent: page_parent(PAGE_B),
                first_block_id: BLOCK_C.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        sqlx::query("UPDATE notes_pages SET archived = 1 WHERE id = ?")
            .bind(PAGE_A)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("UPDATE notes_pages SET in_trash = 1 WHERE id = ?")
            .bind(PAGE_B)
            .execute(&pool)
            .await
            .unwrap();

        let breadcrumb = reads::get_page_breadcrumb(&pool, PAGE_C).await.unwrap();
        let breadcrumb_json = serde_json::to_value(breadcrumb).unwrap();
        assert_eq!(breadcrumb_json[0]["id"], PAGE_A);
        assert_eq!(breadcrumb_json[0]["status"], "archived");
        assert_eq!(breadcrumb_json[1]["id"], PAGE_B);
        assert_eq!(breadcrumb_json[1]["status"], "trashed");
        assert_eq!(breadcrumb_json[2]["id"], PAGE_C);
        assert_eq!(breadcrumb_json[2]["status"], "active");
    });
}

#[test]
fn page_breadcrumb_marks_missing_ancestor_rows() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        sqlx::raw_sql("PRAGMA foreign_keys=OFF")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE notes_pages
             SET parent_type = 'page_id', parent_page_id = ?
             WHERE id = ?",
        )
        .bind(PAGE_B)
        .bind(PAGE_A)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::raw_sql("PRAGMA foreign_keys=ON")
            .execute(&pool)
            .await
            .unwrap();

        let breadcrumb = reads::get_page_breadcrumb(&pool, PAGE_A).await.unwrap();
        let breadcrumb_json = serde_json::to_value(breadcrumb).unwrap();
        assert_eq!(breadcrumb_json.as_array().unwrap().len(), 2);
        assert_eq!(breadcrumb_json[0]["id"], PAGE_B);
        assert_eq!(breadcrumb_json[0]["status"], "missing");
        assert_eq!(breadcrumb_json[1]["id"], PAGE_A);
        assert_eq!(breadcrumb_json[1]["current"], true);
    });
}

#[test]
fn backlinks_include_visible_child_page_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();

        let backlinks = reads::list_backlinks(&pool, PAGE_B).await.unwrap();
        let backlinks_json = serde_json::to_value(backlinks).unwrap();
        assert_eq!(backlinks_json.as_array().unwrap().len(), 1);
        assert_eq!(backlinks_json[0]["object"], "backlink");
        assert_eq!(backlinks_json[0]["source_page"]["id"], PAGE_A);
        assert_eq!(backlinks_json[0]["source_block_id"], PAGE_B);
        assert_eq!(backlinks_json[0]["reference_type"], "child_page");

        writes::archive_page(&pool, PAGE_A, true).await.unwrap();
        assert!(reads::list_backlinks(&pool, PAGE_B)
            .await
            .unwrap()
            .is_empty());
    });
}

#[test]
fn backlinks_include_local_notes_rich_text_links() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_page(&pool, PAGE_B, BLOCK_B).await;
        let linked_payload = json!({
            "rich_text": [{
                "type": "text",
                "text": {
                    "content": "See target",
                    "link": {
                        "url": format!(
                            "http://localhost:1420/?view=notes#notes?page={PAGE_B}&block={BLOCK_B}"
                        )
                    }
                },
                "annotations": {
                    "bold": false,
                    "italic": false,
                    "strikethrough": false,
                    "underline": false,
                    "code": false,
                    "color": "default"
                },
                "plain_text": "See target",
                "href": format!(
                    "http://localhost:1420/?view=notes#notes?page={PAGE_B}&block={BLOCK_B}"
                )
            }],
            "color": "default"
        });
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_C, "paragraph", linked_payload)],
            },
        )
        .await
        .unwrap();

        let backlinks = reads::list_backlinks(&pool, PAGE_B).await.unwrap();
        let backlinks_json = serde_json::to_value(backlinks).unwrap();
        assert_eq!(backlinks_json.as_array().unwrap().len(), 1);
        assert_eq!(backlinks_json[0]["source_page"]["id"], PAGE_A);
        assert_eq!(backlinks_json[0]["source_block_id"], BLOCK_C);
        assert_eq!(backlinks_json[0]["reference_type"], "link");
        assert_eq!(backlinks_json[0]["snippet"], "See target");
    });
}

#[test]
fn backlinks_refresh_when_local_notes_rich_text_links_are_edited() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_page(&pool, PAGE_B, BLOCK_B).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("See target"))],
            },
        )
        .await
        .unwrap();
        assert!(reads::list_backlinks(&pool, PAGE_B)
            .await
            .unwrap()
            .is_empty());

        let target_url =
            format!("http://localhost:1420/?view=notes#notes?page={PAGE_B}&block={BLOCK_B}");
        writes::update_block(
            &pool,
            BLOCK_C,
            block_update(
                "paragraph",
                json!({
                    "rich_text": [linked_rich_text("See target", &target_url)],
                    "color": "default"
                }),
            ),
        )
        .await
        .unwrap();
        let backlinks = reads::list_backlinks(&pool, PAGE_B).await.unwrap();
        let backlinks_json = serde_json::to_value(backlinks).unwrap();
        assert_eq!(backlinks_json.as_array().unwrap().len(), 1);
        assert_eq!(backlinks_json[0]["source_page"]["id"], PAGE_A);
        assert_eq!(backlinks_json[0]["source_block_id"], BLOCK_C);
        assert_eq!(backlinks_json[0]["reference_type"], "link");

        writes::update_block(
            &pool,
            BLOCK_C,
            block_update(
                "paragraph",
                json!({
                    "rich_text": [linked_rich_text("Email", "mailto:team@example.com")],
                    "color": "default"
                }),
            ),
        )
        .await
        .unwrap();
        assert!(reads::list_backlinks(&pool, PAGE_B)
            .await
            .unwrap()
            .is_empty());
    });
}

#[test]
fn backlinks_include_page_mentions() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_page(&pool, PAGE_B, BLOCK_B).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    json!({
                        "rich_text": [
                            rich_text("See "),
                            page_mention(PAGE_B, "Target page")
                        ],
                        "color": "default"
                    }),
                )],
            },
        )
        .await
        .unwrap();

        let backlinks = reads::list_backlinks(&pool, PAGE_B).await.unwrap();
        let backlinks_json = serde_json::to_value(backlinks).unwrap();
        assert_eq!(backlinks_json.as_array().unwrap().len(), 1);
        assert_eq!(backlinks_json[0]["source_page"]["id"], PAGE_A);
        assert_eq!(backlinks_json[0]["source_block_id"], BLOCK_C);
        assert_eq!(backlinks_json[0]["reference_type"], "page_mention");
        assert_eq!(backlinks_json[0]["snippet"], "See Target page");
    });
}

#[test]
fn backlinks_include_data_source_row_page_mentions() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_database(
            &pool,
            DATABASE_A,
            DATA_SOURCE_A,
            DATABASE_VIEW_A,
            "Tasks",
            BLOCK_A,
        )
        .await;
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Target row".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    json!({
                        "rich_text": [
                            rich_text("See "),
                            page_mention(PAGE_B, "Target row")
                        ],
                        "color": "default"
                    }),
                )],
            },
        )
        .await
        .unwrap();

        let backlinks = reads::list_backlinks(&pool, PAGE_B).await.unwrap();
        let backlinks_json = serde_json::to_value(backlinks).unwrap();
        assert_eq!(backlinks_json.as_array().unwrap().len(), 1);
        assert_eq!(backlinks_json[0]["source_page"]["id"], PAGE_A);
        assert_eq!(backlinks_json[0]["source_block_id"], BLOCK_C);
        assert_eq!(backlinks_json[0]["reference_type"], "page_mention");
    });
}

#[test]
fn backlinks_hide_trashed_blocks_and_source_pages() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_page(&pool, PAGE_B, BLOCK_B).await;
        let target_url =
            format!("http://localhost:1420/?view=notes#notes?page={PAGE_B}&block={BLOCK_B}");
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    json!({
                        "rich_text": [linked_rich_text("See target", &target_url)],
                        "color": "default"
                    }),
                )],
            },
        )
        .await
        .unwrap();
        assert_eq!(reads::list_backlinks(&pool, PAGE_B).await.unwrap().len(), 1);

        writes::trash_block(&pool, BLOCK_C, true).await.unwrap();
        assert!(reads::list_backlinks(&pool, PAGE_B)
            .await
            .unwrap()
            .is_empty());

        writes::trash_block(&pool, BLOCK_C, false).await.unwrap();
        assert_eq!(reads::list_backlinks(&pool, PAGE_B).await.unwrap().len(), 1);

        writes::trash_page(&pool, PAGE_A, true).await.unwrap();
        assert!(reads::list_backlinks(&pool, PAGE_B)
            .await
            .unwrap()
            .is_empty());
    });
}

#[test]
fn search_returns_page_block_and_comment_matches() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_A.to_string(),
                title: "Target page".to_string(),
                parent: workspace_parent(),
                first_block_id: BLOCK_A.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();
        create_page(&pool, PAGE_B, BLOCK_B).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_B),
                after: Some(BLOCK_B.to_string()),
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    paragraph_payload("Target block"),
                )],
            },
        )
        .await
        .unwrap();
        comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_A.to_string(),
                parent: Some(page_parent(PAGE_B)),
                discussion_id: None,
                anchor: None,
                rich_text: vec![rich_text("Target comment")],
            },
        )
        .await
        .unwrap();

        let results = reads::search(&pool, "target", Some(10)).await.unwrap();
        let results_json = serde_json::to_value(results).unwrap();
        assert_eq!(results_json.as_array().unwrap().len(), 3);
        assert_eq!(results_json[0]["type"], "page");
        assert_eq!(results_json[0]["page"]["id"], PAGE_A);
        assert_eq!(results_json[1]["type"], "block");
        assert_eq!(results_json[1]["block_id"], BLOCK_C);
        assert_eq!(results_json[2]["type"], "comment");
        assert_eq!(results_json[2]["comment_id"], COMMENT_A);
    });
}

#[test]
fn search_rejects_empty_queries() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        assert_eq!(
            reads::search(&pool, "  ", Some(10)).await.err(),
            Some("search query must not be empty".to_string())
        );
    });
}

#[test]
fn move_page_between_workspace_and_parent_page() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_page(&pool, PAGE_B, BLOCK_B).await;

        let moved = writes::move_page(
            &pool,
            PAGE_B,
            NoteMovePage {
                parent: page_parent(PAGE_A),
            },
        )
        .await
        .unwrap();
        let moved_page = serde_json::to_value(moved).unwrap();
        assert_eq!(moved_page["page"]["parent"]["type"], "page_id");
        assert_eq!(moved_page["page"]["parent"]["page_id"], PAGE_A);

        let parent_blocks = reads::get_block_children(&pool, PAGE_A, None, Some(10))
            .await
            .unwrap();
        let parent_blocks_json = serde_json::to_value(parent_blocks).unwrap();
        assert_eq!(parent_blocks_json["results"][1]["id"], PAGE_B);
        assert_eq!(parent_blocks_json["results"][1]["type"], "child_page");
        assert_eq!(
            parent_blocks_json["results"][1]["child_page"]["title"],
            "First page"
        );

        let moved_top_level = writes::move_page(
            &pool,
            PAGE_B,
            NoteMovePage {
                parent: workspace_parent(),
            },
        )
        .await
        .unwrap();
        let moved_top_level_page = serde_json::to_value(moved_top_level).unwrap();
        assert_eq!(moved_top_level_page["page"]["parent"]["type"], "workspace");
        assert!(reads::get_block(&pool, PAGE_B, false).await.is_err());
    });
}

#[test]
fn move_page_rejects_descendant_parent() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();

        let result = writes::move_page(
            &pool,
            PAGE_A,
            NoteMovePage {
                parent: page_parent(PAGE_B),
            },
        )
        .await;
        let Err(error) = result else {
            panic!("descendant parent unexpectedly accepted a moved page");
        };
        assert_eq!(
            error,
            "page cannot be moved under its descendant".to_string()
        );
    });
}

#[test]
fn archiving_nested_page_hides_child_page_block() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Nested".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
            },
        )
        .await
        .unwrap();

        writes::archive_page(&pool, PAGE_B, true).await.unwrap();
        let parent_blocks = reads::get_block_children(&pool, PAGE_A, None, Some(10))
            .await
            .unwrap();
        let parent_blocks_json = serde_json::to_value(parent_blocks).unwrap();
        assert_eq!(parent_blocks_json["results"].as_array().unwrap().len(), 1);
        assert!(reads::get_block(&pool, PAGE_B, false).await.is_err());

        writes::archive_page(&pool, PAGE_B, false).await.unwrap();
        assert!(reads::get_block(&pool, PAGE_B, false).await.is_ok());
    });
}

#[test]
fn page_icon_set_and_remove_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let page = writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: None,
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Value(json!({
                    "type": "emoji",
                    "emoji": "📌"
                })),
                cover: OptionalJsonValue::Unset,
            },
        )
        .await
        .unwrap();
        let page_json = serde_json::to_value(page).unwrap();
        assert_eq!(page_json["icon"]["type"], "emoji");
        assert_eq!(page_json["icon"]["emoji"], "📌");
        assert!(page_json["cover"].is_null());

        let page = writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: None,
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Null,
                cover: OptionalJsonValue::Unset,
            },
        )
        .await
        .unwrap();
        let page_json = serde_json::to_value(page).unwrap();
        assert!(page_json["icon"].is_null());
        let stored_icon: Option<String> =
            sqlx::query_scalar("SELECT icon FROM notes_pages WHERE id = ?")
                .bind(PAGE_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(stored_icon.is_none());
    });
}

#[test]
fn page_icon_validation_rejects_empty_emoji() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let result = writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: None,
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Value(json!({
                    "type": "emoji",
                    "emoji": ""
                })),
                cover: OptionalJsonValue::Unset,
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("icon.emoji must be a non-empty string".to_string())
        );
    });
}

#[test]
fn page_icon_variants_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let variants = [
            json!({
                "type": "icon",
                "icon": {
                    "name": "home",
                    "color": "blue"
                }
            }),
            json!({
                "type": "custom_emoji",
                "custom_emoji": {
                    "id": "emoji-1",
                    "name": "Focus",
                    "url": "ganbaru-asset:project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
                    "ganbaru_asset_path": "project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png"
                }
            }),
            json!({
                "type": "external",
                "external": {
                    "url": "https://example.com/icon.webp"
                }
            }),
            json!({
                "type": "file",
                "file": {
                    "url": "ganbaru-asset:notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                    "name": "focus.png",
                    "content_type": "image/png",
                    "byte_size": 42,
                    "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    "ganbaru_asset_path": "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png"
                }
            }),
        ];

        for icon in variants {
            let page = writes::update_page(
                &pool,
                PAGE_A,
                super::models::NotePageUpdate {
                    title: None,
                    parent: None,
                    properties: None,
                    icon: OptionalJsonValue::Value(icon.clone()),
                    cover: OptionalJsonValue::Unset,
                },
            )
            .await
            .unwrap();
            let page_json = serde_json::to_value(page).unwrap();
            assert_eq!(page_json["icon"], icon);
        }
    });
}

#[test]
fn page_icon_validation_rejects_unsafe_external_and_local_files() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        for (icon, expected_error) in [
            (
                json!({
                    "type": "external",
                    "external": { "url": "http://example.com/icon.png" }
                }),
                "icon.external.url must be a supported HTTPS image URL",
            ),
            (
                json!({
                    "type": "external",
                    "external": { "url": "https://example.com/icon.svg" }
                }),
                "icon.external.url must be a supported HTTPS image URL",
            ),
            (
                json!({
                    "type": "custom_emoji",
                    "custom_emoji": { "name": "Missing id" }
                }),
                "icon.custom_emoji.id must be a non-empty string",
            ),
            (
                json!({
                    "type": "custom_emoji",
                    "custom_emoji": {
                        "id": "wrong-directory",
                        "url": "ganbaru-asset:notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png"
                    }
                }),
                "icon.custom_emoji.url must stay under a managed image asset directory",
            ),
            (
                json!({
                    "type": "custom_emoji",
                    "custom_emoji": {
                        "id": "missing-asset-path",
                        "url": "ganbaru-asset:project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png"
                    }
                }),
                "icon.custom_emoji.url must reference the managed icon asset path",
            ),
            (
                json!({
                    "type": "file",
                    "file": {
                        "url": "ganbaru://assets/notes/page-icons/bad.svg",
                        "content_type": "image/svg+xml",
                        "byte_size": 42,
                        "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        "ganbaru_asset_path": "notes/page-icons/bad.svg"
                    }
                }),
                "icon.file.ganbaru_asset_path must stay under a managed image asset directory",
            ),
            (
                json!({
                    "type": "file",
                    "file": {
                        "url": "ganbaru-asset:notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                        "content_type": "image/png",
                        "byte_size": 0,
                        "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        "ganbaru_asset_path": "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png"
                    }
                }),
                "icon.file.byte_size must be positive",
            ),
            (
                json!({
                    "type": "file",
                    "file": {
                        "url": "ganbaru-asset:notes/page-icons/cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc.png",
                        "content_type": "image/png",
                        "byte_size": 42,
                        "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        "ganbaru_asset_path": "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png"
                    }
                }),
                "icon.file.url must reference the managed icon asset path",
            ),
            (
                json!({
                    "type": "file",
                    "file": {
                        "url": "ganbaru-asset:notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png"
                    }
                }),
                "icon.file.url must include managed asset metadata",
            ),
        ] {
            let result = writes::update_page(
                &pool,
                PAGE_A,
                super::models::NotePageUpdate {
                    title: None,
                    parent: None,
                    properties: None,
                    icon: OptionalJsonValue::Value(icon),
                    cover: OptionalJsonValue::Unset,
                },
            )
            .await;
            assert_eq!(result.err(), Some(expected_error.to_string()));
        }
    });
}

#[test]
fn page_cover_set_and_remove_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let page = writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: None,
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Unset,
                cover: OptionalJsonValue::Value(json!({
                    "type": "external",
                    "external": {
                        "url": "https://example.com/cover.jpg"
                    }
                })),
            },
        )
        .await
        .unwrap();
        let page_json = serde_json::to_value(page).unwrap();
        assert_eq!(page_json["cover"]["type"], "external");
        assert_eq!(
            page_json["cover"]["external"]["url"],
            "https://example.com/cover.jpg"
        );

        let page = writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: None,
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Unset,
                cover: OptionalJsonValue::Null,
            },
        )
        .await
        .unwrap();
        let page_json = serde_json::to_value(page).unwrap();
        assert!(page_json["cover"].is_null());
    });
}

#[test]
fn page_cover_variants_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let variants = [
            json!({
                "type": "file",
                "file": {
                    "url": "https://example.com/imported-cover.webp",
                    "expiry_time": "2026-07-01T12:00:00.000Z"
                }
            }),
            json!({
                "type": "file_upload",
                "file_upload": {
                    "id": "55555555-5555-4555-8555-555555555555"
                }
            }),
            json!({
                "type": "file",
                "file": {
                    "url": "ganbaru-asset:notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
                    "name": "cover.png",
                    "content_type": "image/png",
                    "byte_size": 42,
                    "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "ganbaru_asset_path": "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png"
                }
            }),
        ];

        for cover in variants {
            let page = writes::update_page(
                &pool,
                PAGE_A,
                super::models::NotePageUpdate {
                    title: None,
                    parent: None,
                    properties: None,
                    icon: OptionalJsonValue::Unset,
                    cover: OptionalJsonValue::Value(cover.clone()),
                },
            )
            .await
            .unwrap();
            let page_json = serde_json::to_value(page).unwrap();
            assert_eq!(page_json["cover"], cover);
        }
    });
}

#[test]
fn page_media_asset_references_follow_local_file_payloads() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        let icon_path =
            "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png";
        let cover_path =
            "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png";

        writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: None,
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Value(json!({
                    "type": "file",
                    "file": {
                        "url": format!("ganbaru-asset:{icon_path}"),
                        "name": "focus.png",
                        "content_type": "image/png",
                        "byte_size": 42,
                        "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        "ganbaru_asset_path": icon_path
                    }
                })),
                cover: OptionalJsonValue::Value(json!({
                    "type": "file",
                    "file": {
                        "url": format!("ganbaru-asset:{cover_path}"),
                        "name": "cover.png",
                        "content_type": "image/png",
                        "byte_size": 84,
                        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "ganbaru_asset_path": cover_path
                    }
                })),
            },
        )
        .await
        .unwrap();

        let asset_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_assets WHERE kind = 'image'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(asset_count, 2);

        for (asset_path, role) in [(icon_path, "page_icon"), (cover_path, "page_cover")] {
            let reference_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM notes_asset_references
                 WHERE asset_id = ? AND owner_type = 'page' AND owner_id = ? AND role = ?",
            )
            .bind(asset_path)
            .bind(PAGE_A)
            .bind(role)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(reference_count, 1);
        }

        assets::mark_managed_asset_storage_state(&pool, icon_path, true, "page icon")
            .await
            .unwrap();
        let missing_icon: (String, Option<String>) = sqlx::query_as(
            "SELECT storage_state, missing_at FROM notes_assets WHERE asset_path = ?",
        )
        .bind(icon_path)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(missing_icon.0, "missing");
        assert!(missing_icon.1.is_some());

        assets::mark_managed_asset_storage_state(&pool, icon_path, false, "page icon")
            .await
            .unwrap();
        let recovered_icon: (String, Option<String>) = sqlx::query_as(
            "SELECT storage_state, missing_at FROM notes_assets WHERE asset_path = ?",
        )
        .bind(icon_path)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(recovered_icon, ("available".to_string(), None));

        writes::update_page(
            &pool,
            PAGE_A,
            super::models::NotePageUpdate {
                title: None,
                parent: None,
                properties: None,
                icon: OptionalJsonValue::Null,
                cover: OptionalJsonValue::Value(json!({
                    "type": "external",
                    "external": {
                        "url": "https://example.com/cover.webp"
                    }
                })),
            },
        )
        .await
        .unwrap();

        let remaining_references: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM notes_asset_references WHERE owner_type = 'page' AND owner_id = ?",
        )
        .bind(PAGE_A)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(remaining_references, 0);

        let retained_assets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes_assets")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(retained_assets, 2);
    });
}

#[test]
fn page_cover_validation_rejects_unsafe_file_objects() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        for (cover, expected_error) in [
            (
                json!({
                    "type": "external",
                    "external": {
                        "url": "https://example.com/document.pdf"
                    }
                }),
                "cover.external.url must be a supported HTTPS image URL",
            ),
            (
                json!({
                    "type": "file",
                    "file": {
                        "url": "https://example.com/document.pdf",
                        "expiry_time": "2026-07-01T12:00:00.000Z"
                    }
                }),
                "cover.file.url must be a supported HTTPS image URL",
            ),
            (
                json!({
                    "type": "file",
                    "file": {
                        "url": "ganbaru-asset:notes/page-covers/bad.svg",
                        "content_type": "image/svg+xml",
                        "byte_size": 42,
                        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "ganbaru_asset_path": "notes/page-covers/bad.svg"
                    }
                }),
                "cover.file.ganbaru_asset_path must stay under a managed image asset directory",
            ),
            (
                json!({
                    "type": "file",
                    "file": {
                        "url": "ganbaru-asset:notes/page-covers/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                        "content_type": "image/png",
                        "byte_size": 42,
                        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "ganbaru_asset_path": "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png"
                    }
                }),
                "cover.file.url must reference the managed cover asset path",
            ),
            (
                json!({
                    "type": "file",
                    "file": {
                        "url": "ganbaru-asset:notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png"
                    }
                }),
                "cover.file.url must include managed asset metadata",
            ),
        ] {
            let result = writes::update_page(
                &pool,
                PAGE_A,
                super::models::NotePageUpdate {
                    title: None,
                    parent: None,
                    properties: None,
                    icon: OptionalJsonValue::Unset,
                    cover: OptionalJsonValue::Value(cover),
                },
            )
            .await;
            assert_eq!(result.err(), Some(expected_error.to_string()));
        }
    });
}

#[test]
fn append_children_paginates_and_preserves_payloads() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![
                    block(BLOCK_B, "to_do", todo_payload("Check", true)),
                    block(BLOCK_C, "code", code_payload("let x = 1;", "typescript")),
                ],
            },
        )
        .await
        .unwrap();

        let page_one = reads::get_block_children(&pool, PAGE_A, None, Some(2))
            .await
            .unwrap();
        assert!(serde_json::to_value(&page_one).unwrap()["has_more"]
            .as_bool()
            .unwrap());
        let next_cursor = serde_json::to_value(&page_one).unwrap()["next_cursor"]
            .as_str()
            .unwrap()
            .to_string();
        let page_two = reads::get_block_children(&pool, PAGE_A, Some(&next_cursor), Some(2))
            .await
            .unwrap();
        let page_two_json = serde_json::to_value(&page_two).unwrap();
        assert_eq!(page_two_json["results"][0]["type"], "code");
        assert_eq!(
            page_two_json["results"][0]["code"]["language"],
            "typescript"
        );
    });
}

#[test]
fn append_children_rejects_non_child_parent_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "code",
                    code_payload("let x = 1;", "typescript"),
                )],
            },
        )
        .await
        .unwrap();

        let result = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("code blocks cannot have children".to_string())
        );
    });
}

#[test]
fn append_children_round_trips_toggle_heading_parent_state() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "heading_2",
                    heading_payload("Details", true, Some(false)),
                )],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();

        let heading = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let heading_json = serde_json::to_value(heading).unwrap();
        assert_eq!(heading_json["type"], "heading_2");
        assert_eq!(heading_json["heading_2"]["is_toggleable"], true);
        assert_eq!(heading_json["heading_2"]["ganbaru_open"], false);
        assert_eq!(heading_json["has_children"], true);

        let children = reads::get_block_children(&pool, BLOCK_B, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        assert_eq!(children_json["results"][0]["id"], BLOCK_C);
    });
}

#[test]
fn append_children_rejects_normal_heading_parent_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "heading_2", paragraph_payload("Details"))],
            },
        )
        .await
        .unwrap();

        let result = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("heading_2 blocks cannot have children".to_string())
        );
    });
}

#[test]
fn append_children_rejects_breadcrumb_parent_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "breadcrumb", json!({}))],
            },
        )
        .await
        .unwrap();

        let result = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("breadcrumb blocks cannot have children".to_string())
        );
    });
}

#[test]
fn append_and_update_breadcrumb_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "breadcrumb", json!({}))],
            },
        )
        .await
        .unwrap();

        writes::update_block(&pool, BLOCK_B, block_update("breadcrumb", json!({})))
            .await
            .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "breadcrumb");
        assert_eq!(stored_json["breadcrumb"], json!({}));
    });
}

#[test]
fn append_children_rejects_table_of_contents_parent_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "table_of_contents",
                    json!({ "color": "default" }),
                )],
            },
        )
        .await
        .unwrap();

        let result = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("table_of_contents blocks cannot have children".to_string())
        );
    });
}

#[test]
fn append_and_update_table_of_contents_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "table_of_contents",
                    json!({ "color": "default" }),
                )],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update("table_of_contents", json!({ "color": "blue_background" })),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "table_of_contents");
        assert_eq!(stored_json["table_of_contents"]["color"], "blue_background");
    });
}

#[test]
fn append_and_update_table_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "table", table_payload(2))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![
                    block(BLOCK_C, "table_row", table_row_payload(&["Name", "Status"])),
                    block(
                        BLOCK_D,
                        "table_row",
                        table_row_payload(&["Ganbaru", "Local"]),
                    ),
                ],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "table",
                json!({
                    "table_width": 3,
                    "has_column_header": true,
                    "has_row_header": true
                }),
            ),
        )
        .await
        .unwrap();
        writes::update_block(
            &pool,
            BLOCK_D,
            block_update("table_row", table_row_payload(&["Ganbaru AI", "Offline"])),
        )
        .await
        .unwrap();

        let table_children = reads::get_block_children(&pool, BLOCK_B, None, Some(10))
            .await
            .unwrap();
        let table_children_json = serde_json::to_value(table_children).unwrap();
        assert_eq!(table_children_json["results"].as_array().unwrap().len(), 2);
        assert_eq!(table_children_json["results"][1]["type"], "table_row");
        let stored_table = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_table_json = serde_json::to_value(stored_table).unwrap();
        assert_eq!(stored_table_json["table"]["table_width"], 3);
        assert_eq!(stored_table_json["table"]["has_column_header"], true);
        assert_eq!(stored_table_json["table"]["has_row_header"], true);
        assert_eq!(
            table_children_json["results"][1]["table_row"]["cells"][0][0]["plain_text"],
            "Ganbaru AI"
        );
        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_D)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "Ganbaru AI\tOffline");
    });
}

#[test]
fn append_children_rejects_invalid_table_children() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let row_under_page = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "table_row", table_row_payload(&["Nope"]))],
            },
        )
        .await;
        assert_eq!(
            row_under_page.err(),
            Some("table_row blocks must be children of table blocks".to_string())
        );

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "table", table_payload(2))],
            },
        )
        .await
        .unwrap();
        let paragraph_under_table = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Nope"))],
            },
        )
        .await;

        assert_eq!(
            paragraph_under_table.err(),
            Some("paragraph blocks cannot be children of table blocks".to_string())
        );
    });
}

#[test]
fn append_and_update_tab_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "tab", tab_payload())],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![
                    block(BLOCK_C, "paragraph", paragraph_icon_payload("Overview")),
                    block(BLOCK_D, "paragraph", paragraph_payload("Details")),
                ],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_C),
                after: None,
                children: vec![block(BLOCK_E, "to_do", todo_payload("Read notes", false))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_D),
                after: None,
                children: vec![block(BLOCK_F, "paragraph", paragraph_payload("Draft"))],
            },
        )
        .await
        .unwrap();

        writes::update_block(&pool, BLOCK_B, block_update("tab", tab_payload()))
            .await
            .unwrap();
        writes::update_block(
            &pool,
            BLOCK_C,
            block_update(
                "paragraph",
                json!({
                    "rich_text": [rich_text("Plan")],
                    "color": "default",
                    "icon": {
                        "type": "emoji",
                        "emoji": "✅"
                    }
                }),
            ),
        )
        .await
        .unwrap();
        writes::move_block(
            &pool,
            BLOCK_E,
            NoteMoveBlock {
                parent: block_parent(BLOCK_D),
                after: Some(BLOCK_F.to_string()),
                before: None,
            },
        )
        .await
        .unwrap();
        writes::move_block(
            &pool,
            BLOCK_D,
            NoteMoveBlock {
                parent: block_parent(BLOCK_B),
                after: None,
                before: Some(BLOCK_C.to_string()),
            },
        )
        .await
        .unwrap();

        let tab_children = reads::get_block_children(&pool, BLOCK_B, None, Some(10))
            .await
            .unwrap();
        let tab_children_json = serde_json::to_value(tab_children).unwrap();
        assert_eq!(tab_children_json["results"].as_array().unwrap().len(), 2);
        assert_eq!(tab_children_json["results"][0]["type"], "paragraph");
        assert_eq!(
            tab_children_json["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Details"
        );
        assert_eq!(
            tab_children_json["results"][1]["paragraph"]["rich_text"][0]["plain_text"],
            "Plan"
        );
        assert_eq!(
            tab_children_json["results"][1]["paragraph"]["icon"]["emoji"],
            "✅"
        );

        let first_tab_panel = reads::get_block_children(&pool, BLOCK_D, None, Some(10))
            .await
            .unwrap();
        let first_tab_panel_json = serde_json::to_value(first_tab_panel).unwrap();
        assert_eq!(first_tab_panel_json["results"][0]["type"], "paragraph");
        assert_eq!(
            first_tab_panel_json["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Draft"
        );
        assert_eq!(first_tab_panel_json["results"][1]["type"], "to_do");
        assert_eq!(
            first_tab_panel_json["results"][1]["to_do"]["rich_text"][0]["plain_text"],
            "Read notes"
        );

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "");
    });
}

#[test]
fn append_children_rejects_invalid_tab_children() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "tab", tab_payload())],
            },
        )
        .await
        .unwrap();

        let todo_under_tab = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "to_do", todo_payload("Nope", false))],
            },
        )
        .await;
        assert_eq!(
            todo_under_tab.err(),
            Some("to_do blocks cannot be children of tab blocks".to_string())
        );

        let icon_paragraph_under_page = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_B.to_string()),
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    paragraph_icon_payload("Icon outside tab"),
                )],
            },
        )
        .await;
        assert_eq!(
            icon_paragraph_under_page.err(),
            Some("paragraph.icon is only supported for tab labels".to_string())
        );

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    paragraph_icon_payload("Overview"),
                )],
            },
        )
        .await
        .unwrap();
        let moved_icon_label = writes::move_block(
            &pool,
            BLOCK_C,
            NoteMoveBlock {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_B.to_string()),
                before: None,
            },
        )
        .await;
        assert_eq!(
            moved_icon_label.err(),
            Some("paragraph.icon is only supported for tab labels".to_string())
        );
    });
}

#[test]
fn update_blocks_rejects_tab_child_shape_breaks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "to_do", todo_payload("Child", false))],
            },
        )
        .await
        .unwrap();

        let result = writes::update_block(&pool, BLOCK_B, block_update("tab", tab_payload())).await;

        assert_eq!(
            result.err(),
            Some("tab blocks can only contain paragraph blocks".to_string())
        );

        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "tab", tab_payload())],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Overview"))],
            },
        )
        .await
        .unwrap();

        let result = writes::update_block(
            &pool,
            BLOCK_B,
            block_update("paragraph", paragraph_payload("")),
        )
        .await;

        assert_eq!(
            result.err(),
            Some("tab blocks with labels cannot be converted to paragraph blocks".to_string())
        );
    });
}

#[test]
fn update_blocks_rejects_table_child_shape_breaks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();

        let result =
            writes::update_block(&pool, BLOCK_B, block_update("table", table_payload(2))).await;

        assert_eq!(
            result.err(),
            Some("table blocks can only contain table_row blocks".to_string())
        );
    });
}

#[test]
fn update_blocks_rejects_toggle_heading_child_shape_breaks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "heading_1",
                    heading_payload("Parent", true, Some(true)),
                )],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();

        let result = writes::update_block(
            &pool,
            BLOCK_B,
            block_update("heading_1", paragraph_payload("Parent")),
        )
        .await;

        assert_eq!(
            result.err(),
            Some("heading_1 blocks with children must stay toggleable".to_string())
        );
    });
}

#[test]
fn append_and_read_column_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "column_list", json!({}))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![
                    block(BLOCK_C, "column", column_payload(Some(0.5))),
                    block(BLOCK_D, "column", column_payload(Some(0.5))),
                ],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_C),
                after: None,
                children: vec![block(BLOCK_E, "paragraph", paragraph_payload("Left"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_D),
                after: None,
                children: vec![block(BLOCK_F, "paragraph", paragraph_payload("Right"))],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_C,
            block_update("column", column_payload(Some(0.7))),
        )
        .await
        .unwrap();
        writes::update_block(
            &pool,
            BLOCK_D,
            block_update("column", column_payload(Some(0.3))),
        )
        .await
        .unwrap();
        writes::move_block(
            &pool,
            BLOCK_F,
            NoteMoveBlock {
                parent: block_parent(BLOCK_C),
                after: Some(BLOCK_E.to_string()),
                before: None,
            },
        )
        .await
        .unwrap();

        let column_list_children = reads::get_block_children(&pool, BLOCK_B, None, Some(10))
            .await
            .unwrap();
        let column_list_children_json = serde_json::to_value(column_list_children).unwrap();
        assert_eq!(
            column_list_children_json["results"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(column_list_children_json["results"][0]["type"], "column");
        assert_eq!(
            column_list_children_json["results"][0]["column"]["width_ratio"],
            0.7
        );
        assert_eq!(
            column_list_children_json["results"][1]["column"]["width_ratio"],
            0.3
        );

        let left_column_children = reads::get_block_children(&pool, BLOCK_C, None, Some(10))
            .await
            .unwrap();
        let left_column_children_json = serde_json::to_value(left_column_children).unwrap();
        assert_eq!(left_column_children_json["results"][0]["type"], "paragraph");
        assert_eq!(
            left_column_children_json["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Left"
        );
        assert_eq!(
            left_column_children_json["results"][1]["paragraph"]["rich_text"][0]["plain_text"],
            "Right"
        );
    });
}

#[test]
fn append_children_rejects_invalid_column_children() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let column_under_page = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "column", column_payload(None))],
            },
        )
        .await;
        assert_eq!(
            column_under_page.err(),
            Some("column blocks must be children of column_list blocks".to_string())
        );

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "column_list", json!({}))],
            },
        )
        .await
        .unwrap();
        let paragraph_under_column_list = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Nope"))],
            },
        )
        .await;

        assert_eq!(
            paragraph_under_column_list.err(),
            Some("paragraph blocks cannot be children of column_list blocks".to_string())
        );
    });
}

#[test]
fn update_blocks_rejects_column_list_child_shape_breaks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();

        let result =
            writes::update_block(&pool, BLOCK_B, block_update("column_list", json!({}))).await;

        assert_eq!(
            result.err(),
            Some("column_list blocks can only contain column blocks".to_string())
        );
    });
}

#[test]
fn append_children_rejects_bookmark_parent_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "bookmark",
                    bookmark_payload("https://example.com", ""),
                )],
            },
        )
        .await
        .unwrap();

        let result = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("bookmark blocks cannot have children".to_string())
        );
    });
}

#[test]
fn append_and_update_bookmark_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "bookmark",
                    bookmark_payload("https://example.com", "Reference"),
                )],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "bookmark",
                bookmark_payload("https://example.com/updated", "Updated"),
            ),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "bookmark");
        assert_eq!(
            stored_json["bookmark"]["url"],
            "https://example.com/updated"
        );
        assert_eq!(
            stored_json["bookmark"]["caption"][0]["plain_text"],
            "Updated"
        );

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "Updated https://example.com/updated");
    });
}

#[test]
fn append_children_rejects_link_preview_parent_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "link_preview",
                    link_preview_payload("https://github.com/example/repo/pull/123"),
                )],
            },
        )
        .await
        .unwrap();

        let result = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("link_preview blocks cannot have children".to_string())
        );
    });
}

#[test]
fn append_and_update_link_preview_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "link_preview",
                    link_preview_payload("https://example.com"),
                )],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "link_preview",
                link_preview_payload("https://github.com/example/repo/pull/123"),
            ),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "link_preview");
        assert_eq!(
            stored_json["link_preview"]["url"],
            "https://github.com/example/repo/pull/123"
        );

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "https://github.com/example/repo/pull/123");
    });
}

#[test]
fn append_original_synced_block_accepts_children() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "synced_block",
                    synced_block_payload_original(),
                )],
            },
        )
        .await
        .unwrap();

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    paragraph_payload("Synced child"),
                )],
            },
        )
        .await
        .unwrap();

        let children = reads::get_block_children(&pool, BLOCK_B, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        assert_eq!(children_json["results"][0]["type"], "paragraph");
        assert_eq!(
            children_json["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Synced child"
        );
    });
}

#[test]
fn append_children_rejects_duplicate_synced_block_parent_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "synced_block",
                    synced_block_payload_duplicate(BLOCK_A),
                )],
            },
        )
        .await
        .unwrap();

        let result = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("synced_block blocks cannot have children".to_string())
        );
    });
}

#[test]
fn append_and_update_synced_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "synced_block",
                    synced_block_payload_original(),
                )],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update("synced_block", synced_block_payload_duplicate(BLOCK_A)),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "synced_block");
        assert_eq!(
            stored_json["synced_block"]["synced_from"]["block_id"],
            BLOCK_A
        );

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, BLOCK_A);
    });
}

#[test]
fn update_synced_block_with_children_rejects_duplicate_payload() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "synced_block",
                    synced_block_payload_original(),
                )],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();

        let result = writes::update_block(
            &pool,
            BLOCK_B,
            block_update("synced_block", synced_block_payload_duplicate(BLOCK_A)),
        )
        .await;

        assert_eq!(
            result.err(),
            Some("synced_block blocks with children must stay original".to_string())
        );
    });
}

#[test]
fn append_and_update_heading_4_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "heading_4",
                    heading_payload("Details", true, Some(false)),
                )],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();
        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "heading_4",
                heading_payload("Updated details", true, Some(true)),
            ),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "heading_4");
        assert_eq!(stored_json["has_children"], true);
        assert_eq!(
            stored_json["heading_4"]["rich_text"][0]["plain_text"],
            "Updated details"
        );
        assert_eq!(stored_json["heading_4"]["is_toggleable"], true);
        assert_eq!(stored_json["heading_4"]["ganbaru_open"], true);

        let children = reads::get_block_children(&pool, BLOCK_B, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        assert_eq!(children_json["results"][0]["id"], BLOCK_C);

        let result = writes::update_block(
            &pool,
            BLOCK_B,
            block_update("heading_4", paragraph_payload("Not toggleable")),
        )
        .await;
        assert_eq!(
            result.err(),
            Some("heading_4 blocks with children must stay toggleable".to_string())
        );

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "Updated details");
    });
}

#[test]
fn append_and_update_child_database_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "child_database",
                    child_database_payload("Tasks"),
                )],
            },
        )
        .await
        .unwrap();

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    paragraph_payload("Database child"),
                )],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update("child_database", child_database_payload("Roadmap")),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "child_database");
        assert_eq!(stored_json["has_children"], true);
        assert_eq!(stored_json["child_database"]["title"], "Roadmap");

        let children = reads::get_block_children(&pool, BLOCK_B, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        assert_eq!(children_json["results"][0]["type"], "paragraph");
        assert_eq!(
            children_json["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Database child"
        );

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "Roadmap");
    });
}

#[test]
fn append_update_and_duplicate_template_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "template", template_payload("Add task"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(
                    BLOCK_C,
                    "to_do",
                    todo_payload("Template item", false),
                )],
            },
        )
        .await
        .unwrap();
        writes::update_block(
            &pool,
            BLOCK_B,
            block_update("template", template_payload("Plan day")),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "template");
        assert_eq!(stored_json["has_children"], true);
        assert_eq!(
            stored_json["template"]["rich_text"][0]["plain_text"],
            "Plan day"
        );

        writes::duplicate_block(
            &pool,
            BLOCK_B,
            NoteDuplicateBlock {
                duplicated_block_ids: vec![
                    NoteDuplicatedBlockId {
                        source_id: BLOCK_B.to_string(),
                        duplicate_id: BLOCK_E.to_string(),
                    },
                    NoteDuplicatedBlockId {
                        source_id: BLOCK_C.to_string(),
                        duplicate_id: BLOCK_F.to_string(),
                    },
                ],
            },
        )
        .await
        .unwrap();

        let duplicate = reads::get_block(&pool, BLOCK_E, false).await.unwrap();
        let duplicate_json = serde_json::to_value(duplicate).unwrap();
        assert_eq!(duplicate_json["type"], "template");
        assert_eq!(
            duplicate_json["template"]["rich_text"][0]["plain_text"],
            "Plan day"
        );
        assert_eq!(duplicate_json["has_children"], true);

        let duplicate_children = reads::get_block_children(&pool, BLOCK_E, None, Some(10))
            .await
            .unwrap();
        let duplicate_children_json = serde_json::to_value(duplicate_children).unwrap();
        assert_eq!(duplicate_children_json["results"][0]["id"], BLOCK_F);
        assert_eq!(duplicate_children_json["results"][0]["type"], "to_do");

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "Plan day");
    });
}

#[test]
fn append_update_and_duplicate_button_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "button", button_payload("Add agenda"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(
                    BLOCK_C,
                    "paragraph",
                    paragraph_payload("Generated agenda item"),
                )],
            },
        )
        .await
        .unwrap();
        writes::update_block(
            &pool,
            BLOCK_B,
            block_update("button", button_payload("Add checklist")),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "button");
        assert_eq!(stored_json["has_children"], true);
        assert_eq!(
            stored_json["button"]["rich_text"][0]["plain_text"],
            "Add checklist"
        );
        assert_eq!(
            stored_json["button"]["actions"][0]["position"],
            "below_button"
        );

        writes::duplicate_block(
            &pool,
            BLOCK_B,
            NoteDuplicateBlock {
                duplicated_block_ids: vec![
                    NoteDuplicatedBlockId {
                        source_id: BLOCK_B.to_string(),
                        duplicate_id: BLOCK_E.to_string(),
                    },
                    NoteDuplicatedBlockId {
                        source_id: BLOCK_C.to_string(),
                        duplicate_id: BLOCK_F.to_string(),
                    },
                ],
            },
        )
        .await
        .unwrap();

        let duplicate = reads::get_block(&pool, BLOCK_E, false).await.unwrap();
        let duplicate_json = serde_json::to_value(duplicate).unwrap();
        assert_eq!(duplicate_json["type"], "button");
        assert_eq!(
            duplicate_json["button"]["rich_text"][0]["plain_text"],
            "Add checklist"
        );
        assert_eq!(duplicate_json["has_children"], true);

        let duplicate_children = reads::get_block_children(&pool, BLOCK_E, None, Some(10))
            .await
            .unwrap();
        let duplicate_children_json = serde_json::to_value(duplicate_children).unwrap();
        assert_eq!(duplicate_children_json["results"][0]["id"], BLOCK_F);
        assert_eq!(duplicate_children_json["results"][0]["type"], "paragraph");

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "Add checklist");
    });
}

#[test]
fn append_children_rejects_embed_parent_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "embed",
                    embed_payload("https://example.com"),
                )],
            },
        )
        .await
        .unwrap();

        let result = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("embed blocks cannot have children".to_string())
        );
    });
}

#[test]
fn append_and_update_embed_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "embed",
                    embed_payload("https://example.com"),
                )],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "embed",
                embed_payload("https://player.vimeo.com/video/226053498"),
            ),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "embed");
        assert_eq!(
            stored_json["embed"]["url"],
            "https://player.vimeo.com/video/226053498"
        );

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "https://player.vimeo.com/video/226053498");
    });
}

#[test]
fn append_and_update_media_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![
                    block(
                        BLOCK_B,
                        "image",
                        media_payload("https://example.com/image.png", "Cover", None),
                    ),
                    block(
                        BLOCK_C,
                        "video",
                        media_payload("https://www.youtube.com/watch?v=abc123", "", None),
                    ),
                    block(
                        BLOCK_D,
                        "audio",
                        media_payload("https://example.com/song.mp3", "", None),
                    ),
                    block(
                        BLOCK_E,
                        "file",
                        media_payload("https://example.com/doc.txt", "Spec", Some("doc.txt")),
                    ),
                    block(
                        BLOCK_F,
                        "pdf",
                        media_payload("https://example.com/doc.pdf", "", None),
                    ),
                ],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "image",
                media_payload("https://example.com/updated.jpg", "Updated cover", None),
            ),
        )
        .await
        .unwrap();

        let stored_image = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_image_json = serde_json::to_value(stored_image).unwrap();
        assert_eq!(stored_image_json["type"], "image");
        assert_eq!(
            stored_image_json["image"]["external"]["url"],
            "https://example.com/updated.jpg"
        );
        assert_eq!(
            stored_image_json["image"]["caption"][0]["plain_text"],
            "Updated cover"
        );

        let file_plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_E)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(file_plain_text, "Spec doc.txt https://example.com/doc.txt");
    });
}

#[test]
fn block_media_asset_references_follow_local_file_payloads() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        let first_asset =
            "notes/files/cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc.png";
        let second_asset =
            "notes/files/dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd.png";

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "image",
                    local_media_payload(
                        first_asset,
                        "image/png",
                        42,
                        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                        "Local cover",
                        Some("cover.png"),
                    ),
                )],
            },
        )
        .await
        .unwrap();

        let first_reference_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notes_asset_references
             WHERE asset_id = ? AND owner_type = 'block' AND owner_id = ?
                AND page_id = ? AND block_id = ? AND role = 'block_file'",
        )
        .bind(first_asset)
        .bind(BLOCK_B)
        .bind(PAGE_A)
        .bind(BLOCK_B)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(first_reference_count, 1);

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "image",
                local_media_payload(
                    second_asset,
                    "image/png",
                    84,
                    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
                    "Replacement",
                    Some("replacement.png"),
                ),
            ),
        )
        .await
        .unwrap();

        let old_reference_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_asset_references WHERE asset_id = ?")
                .bind(first_asset)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(old_reference_count, 0);

        let replacement_reference_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notes_asset_references
             WHERE asset_id = ? AND owner_type = 'block' AND owner_id = ?
                AND page_id = ? AND block_id = ? AND role = 'block_file'",
        )
        .bind(second_asset)
        .bind(BLOCK_B)
        .bind(PAGE_A)
        .bind(BLOCK_B)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(replacement_reference_count, 1);

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "image",
                media_payload("https://example.com/replacement.png", "External", None),
            ),
        )
        .await
        .unwrap();

        let remaining_references: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_asset_references WHERE owner_id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(remaining_references, 0);

        let retained_assets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notes_assets")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(retained_assets, 2);
    });
}

#[test]
fn append_children_rejects_equation_parent_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "equation", equation_payload("e=mc^2"))],
            },
        )
        .await
        .unwrap();

        let result = writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("equation blocks cannot have children".to_string())
        );
    });
}

#[test]
fn append_and_update_equation_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "equation", equation_payload("e=mc^2"))],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update("equation", equation_payload("\\frac{a}{b}")),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "equation");
        assert_eq!(stored_json["equation"]["expression"], "\\frac{a}{b}");

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "\\frac{a}{b}");
    });
}

#[test]
fn append_and_update_unsupported_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "unsupported", unsupported_payload("form"))],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "unsupported",
                json!({
                    "block_type": "button",
                    "source_type": "notion",
                    "raw": {
                        "type": "unsupported",
                        "unsupported": {
                            "block_type": "button"
                        }
                    },
                    "warnings": ["Action content is not exposed"]
                }),
            ),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "unsupported");
        assert_eq!(stored_json["unsupported"]["block_type"], "button");
        assert_eq!(
            stored_json["unsupported"]["raw"]["unsupported"]["block_type"],
            "button"
        );

        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            plain_text,
            "button notion Action content is not exposed raw payload preserved"
        );
    });
}

#[test]
fn update_block_persists_supported_block_color() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::update_block(
            &pool,
            BLOCK_A,
            block_update(
                "paragraph",
                json!({
                    "rich_text": [rich_text("Colored")],
                    "color": "red_background"
                }),
            ),
        )
        .await
        .unwrap();

        let payload: String = sqlx::query_scalar("SELECT payload FROM notes_blocks WHERE id = ?")
            .bind(BLOCK_A)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(payload.contains("\"color\":\"red_background\""));
        assert!(payload.contains("Colored"));
    });
}

#[test]
fn update_block_round_trips_inline_formatting_annotations() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::update_block(
            &pool,
            BLOCK_A,
            block_update(
                "paragraph",
                json!({
                    "rich_text": [{
                        "type": "text",
                        "text": {
                            "content": "Shortcut text",
                            "link": null
                        },
                        "annotations": {
                            "bold": true,
                            "italic": true,
                            "strikethrough": true,
                            "underline": true,
                            "code": true,
                            "color": "default"
                        },
                        "plain_text": "Shortcut text",
                        "href": null
                    }],
                    "color": "default"
                }),
            ),
        )
        .await
        .unwrap();

        let children = reads::get_block_children(&pool, PAGE_A, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        let annotations = &children_json["results"][0]["paragraph"]["rich_text"][0]["annotations"];
        assert_eq!(annotations["bold"], true);
        assert_eq!(annotations["italic"], true);
        assert_eq!(annotations["strikethrough"], true);
        assert_eq!(annotations["underline"], true);
        assert_eq!(annotations["code"], true);
        assert_eq!(
            children_json["results"][0]["paragraph"]["rich_text"][0]["plain_text"],
            "Shortcut text"
        );
    });
}

#[test]
fn rich_text_paste_payloads_round_trip_current_and_appended_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::update_block(
            &pool,
            BLOCK_A,
            block_update(
                "paragraph",
                json!({
                    "rich_text": [
                        rich_text("Before "),
                        annotated_rich_text("styled", "blue_background"),
                        linked_rich_text(" docs", "https://example.com/docs")
                    ],
                    "color": "default"
                }),
            ),
        )
        .await
        .unwrap();

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "paragraph",
                    json!({
                        "rich_text": [
                            rich_text("Next "),
                            linked_rich_text("reference", "mailto:team@example.com")
                        ],
                        "color": "default"
                    }),
                )],
            },
        )
        .await
        .unwrap();

        let children = reads::get_block_children(&pool, PAGE_A, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        let first_rich_text = &children_json["results"][0]["paragraph"]["rich_text"];
        assert_eq!(first_rich_text[0]["plain_text"], "Before ");
        assert_eq!(first_rich_text[1]["annotations"]["bold"], true);
        assert_eq!(
            first_rich_text[1]["annotations"]["color"],
            "blue_background"
        );
        assert_eq!(
            first_rich_text[2]["text"]["link"]["url"],
            "https://example.com/docs"
        );
        assert_eq!(first_rich_text[2]["href"], "https://example.com/docs");

        let second_rich_text = &children_json["results"][1]["paragraph"]["rich_text"];
        assert_eq!(second_rich_text[0]["plain_text"], "Next ");
        assert_eq!(
            second_rich_text[1]["text"]["link"]["url"],
            "mailto:team@example.com"
        );
        assert_eq!(second_rich_text[1]["href"], "mailto:team@example.com");

        let stored_plain_text: Vec<String> = sqlx::query_scalar(
            "SELECT plain_text FROM notes_blocks WHERE id IN (?, ?) ORDER BY sort_order",
        )
        .bind(BLOCK_A)
        .bind(BLOCK_B)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            stored_plain_text,
            vec![
                "Before styled docs".to_string(),
                "Next reference".to_string()
            ]
        );
    });
}

#[test]
fn update_block_round_trips_inline_equations() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::update_block(
            &pool,
            BLOCK_A,
            block_update(
                "paragraph",
                json!({
                    "rich_text": [
                        rich_text("Use "),
                        inline_equation_rich_text("\\frac{a}{b}"),
                        rich_text(" here")
                    ],
                    "color": "default"
                }),
            ),
        )
        .await
        .unwrap();

        let children = reads::get_block_children(&pool, PAGE_A, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        let rich_text = &children_json["results"][0]["paragraph"]["rich_text"];
        assert_eq!(rich_text[0]["plain_text"], "Use ");
        assert_eq!(rich_text[1]["type"], "equation");
        assert_eq!(rich_text[1]["equation"]["expression"], "\\frac{a}{b}");
        assert_eq!(rich_text[1]["plain_text"], "\\frac{a}{b}");
        assert_eq!(rich_text[1]["href"], serde_json::Value::Null);
        assert_eq!(rich_text[2]["plain_text"], " here");
        let plain_text: String =
            sqlx::query_scalar("SELECT plain_text FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(plain_text, "Use \\frac{a}{b} here");
    });
}

#[test]
fn append_and_update_toggle_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "toggle",
                    json!({
                        "rich_text": [rich_text("Details")],
                        "color": "blue_background",
                        "ganbaru_open": true
                    }),
                )],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "toggle",
                json!({
                    "rich_text": [rich_text("Details")],
                    "color": "blue_background",
                    "ganbaru_open": false
                }),
            ),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "toggle");
        assert_eq!(stored_json["toggle"]["ganbaru_open"], false);
        assert_eq!(stored_json["toggle"]["color"], "blue_background");
    });
}

#[test]
fn append_and_update_callout_blocks_round_trip() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "callout",
                    json!({
                        "rich_text": [rich_text("Remember this")],
                        "color": "yellow_background",
                        "icon": {
                            "type": "icon",
                            "icon": {
                                "name": "info",
                                "color": "gray"
                            }
                        }
                    }),
                )],
            },
        )
        .await
        .unwrap();

        writes::update_block(
            &pool,
            BLOCK_B,
            block_update(
                "callout",
                json!({
                    "rich_text": [rich_text("Updated")],
                    "color": "blue_background",
                    "icon": {
                        "type": "icon",
                        "icon": {
                            "name": "info",
                            "color": "blue"
                        }
                    }
                }),
            ),
        )
        .await
        .unwrap();

        let stored = reads::get_block(&pool, BLOCK_B, false).await.unwrap();
        let stored_json = serde_json::to_value(stored).unwrap();
        assert_eq!(stored_json["type"], "callout");
        assert_eq!(stored_json["callout"]["color"], "blue_background");
        assert_eq!(stored_json["callout"]["icon"]["icon"]["color"], "blue");
        assert_eq!(
            stored_json["callout"]["rich_text"][0]["plain_text"],
            "Updated"
        );
    });
}

#[test]
fn move_block_nests_and_outdents_with_parent_state() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![
                    block(BLOCK_B, "paragraph", paragraph_payload("Parent")),
                    block(BLOCK_C, "paragraph", paragraph_payload("Child")),
                ],
            },
        )
        .await
        .unwrap();

        writes::move_block(
            &pool,
            BLOCK_C,
            NoteMoveBlock {
                parent: block_parent(BLOCK_B),
                after: None,
                before: None,
            },
        )
        .await
        .unwrap();
        let nested_parent: String =
            sqlx::query_scalar("SELECT parent_block_id FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_C)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(nested_parent, BLOCK_B);
        let has_children: i64 =
            sqlx::query_scalar("SELECT has_children FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(has_children, 1);

        writes::move_block(
            &pool,
            BLOCK_C,
            NoteMoveBlock {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_B.to_string()),
                before: None,
            },
        )
        .await
        .unwrap();
        let row = sqlx::query("SELECT parent_type, parent_page_id FROM notes_blocks WHERE id = ?")
            .bind(BLOCK_C)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.get::<String, _>("parent_type"), "page_id");
        assert_eq!(row.get::<String, _>("parent_page_id"), PAGE_A);
    });
}

#[test]
fn move_block_can_place_before_first_sibling() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![
                    block(BLOCK_B, "paragraph", paragraph_payload("Second")),
                    block(BLOCK_C, "paragraph", paragraph_payload("Third")),
                    block(BLOCK_D, "paragraph", paragraph_payload("Fourth")),
                ],
            },
        )
        .await
        .unwrap();

        writes::move_block(
            &pool,
            BLOCK_D,
            NoteMoveBlock {
                parent: page_parent(PAGE_A),
                after: None,
                before: Some(BLOCK_A.to_string()),
            },
        )
        .await
        .unwrap();

        let ids = sqlx::query_scalar::<_, String>(
            "SELECT id FROM notes_blocks WHERE parent_page_id = ? ORDER BY sort_order ASC, id ASC",
        )
        .bind(PAGE_A)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(ids, vec![BLOCK_D, BLOCK_A, BLOCK_B, BLOCK_C]);
    });
}

#[test]
fn move_block_rejects_ambiguous_after_and_before() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![
                    block(BLOCK_B, "paragraph", paragraph_payload("Second")),
                    block(BLOCK_C, "paragraph", paragraph_payload("Third")),
                ],
            },
        )
        .await
        .unwrap();

        let result = writes::move_block(
            &pool,
            BLOCK_B,
            NoteMoveBlock {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                before: Some(BLOCK_C.to_string()),
            },
        )
        .await;
        assert_eq!(
            result.err(),
            Some("move request cannot include both after and before".to_string())
        );
    });
}

#[test]
fn move_block_rejects_after_anchor_outside_destination_parent() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![
                    block(BLOCK_B, "paragraph", paragraph_payload("Parent")),
                    block(BLOCK_C, "paragraph", paragraph_payload("Nested")),
                ],
            },
        )
        .await
        .unwrap();
        writes::move_block(
            &pool,
            BLOCK_C,
            NoteMoveBlock {
                parent: block_parent(BLOCK_B),
                after: None,
                before: None,
            },
        )
        .await
        .unwrap();

        let result = writes::move_block(
            &pool,
            BLOCK_A,
            NoteMoveBlock {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_C.to_string()),
                before: None,
            },
        )
        .await;
        assert_eq!(result.err(), Some("after block not found".to_string()));
    });
}

#[test]
fn move_block_rejects_invalid_structural_parent_shapes() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![
                    block(BLOCK_B, "column_list", json!({})),
                    block(BLOCK_C, "table", table_payload(2)),
                    block(BLOCK_D, "paragraph", paragraph_payload("Paragraph")),
                ],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_E, "column", column_payload(Some(1.0)))],
            },
        )
        .await
        .unwrap();

        let paragraph_under_columns = writes::move_block(
            &pool,
            BLOCK_D,
            NoteMoveBlock {
                parent: block_parent(BLOCK_B),
                after: None,
                before: None,
            },
        )
        .await;
        assert_eq!(
            paragraph_under_columns.err(),
            Some("paragraph blocks cannot be children of column_list blocks".to_string())
        );

        let paragraph_under_table = writes::move_block(
            &pool,
            BLOCK_D,
            NoteMoveBlock {
                parent: block_parent(BLOCK_C),
                after: None,
                before: None,
            },
        )
        .await;
        assert_eq!(
            paragraph_under_table.err(),
            Some("paragraph blocks cannot be children of table blocks".to_string())
        );

        let column_under_page = writes::move_block(
            &pool,
            BLOCK_E,
            NoteMoveBlock {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_D.to_string()),
                before: None,
            },
        )
        .await;
        assert_eq!(
            column_under_page.err(),
            Some("column blocks must be children of column_list blocks".to_string())
        );
    });
}

#[test]
fn move_block_to_another_page_updates_descendant_page_ids() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        create_page(&pool, PAGE_B, BLOCK_D).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();

        writes::move_block(
            &pool,
            BLOCK_B,
            NoteMoveBlock {
                parent: page_parent(PAGE_B),
                after: None,
                before: None,
            },
        )
        .await
        .unwrap();

        let rows = sqlx::query("SELECT id, page_id, parent_type, parent_page_id, parent_block_id FROM notes_blocks WHERE id IN (?, ?) ORDER BY id ASC")
            .bind(BLOCK_B)
            .bind(BLOCK_C)
            .fetch_all(&pool)
            .await
            .unwrap();
        let moved_parent = rows
            .iter()
            .find(|row| row.get::<String, _>("id") == BLOCK_B)
            .expect("moved parent row should exist");
        assert_eq!(moved_parent.get::<String, _>("page_id"), PAGE_B);
        assert_eq!(moved_parent.get::<String, _>("parent_type"), "page_id");
        assert_eq!(moved_parent.get::<String, _>("parent_page_id"), PAGE_B);
        let moved_child = rows
            .iter()
            .find(|row| row.get::<String, _>("id") == BLOCK_C)
            .expect("moved child row should exist");
        assert_eq!(moved_child.get::<String, _>("page_id"), PAGE_B);
        assert_eq!(moved_child.get::<String, _>("parent_type"), "block_id");
        assert_eq!(moved_child.get::<String, _>("parent_block_id"), BLOCK_B);
    });
}

#[test]
fn move_block_rejects_destination_page_inside_source_subtree() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::create_child_page_from_block(
            &pool,
            BLOCK_B,
            NoteChildPageFromBlockCreate {
                first_block_id: BLOCK_D.to_string(),
                title: Some("Nested".to_string()),
            },
        )
        .await
        .unwrap();

        let result = writes::move_block(
            &pool,
            BLOCK_B,
            NoteMoveBlock {
                parent: page_parent(BLOCK_B),
                after: None,
                before: None,
            },
        )
        .await;
        assert_eq!(
            result.err(),
            Some("block cannot be moved into a page contained by its subtree".to_string())
        );
    });
}

#[test]
fn trash_block_hides_descendants_and_refreshes_parent() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_D, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();

        writes::trash_block(&pool, BLOCK_B, true).await.unwrap();
        let trashed_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_blocks WHERE in_trash = 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(trashed_count, 2);
        let visible_children = reads::get_block_children(&pool, PAGE_A, None, Some(50))
            .await
            .unwrap();
        let json = serde_json::to_value(visible_children).unwrap();
        assert_eq!(json["results"].as_array().unwrap().len(), 1);
    });
}

#[test]
fn move_block_rejects_descendant_parent() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_C, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();

        let result = writes::move_block(
            &pool,
            BLOCK_B,
            NoteMoveBlock {
                parent: block_parent(BLOCK_C),
                after: None,
                before: None,
            },
        )
        .await;
        assert_eq!(
            result.err(),
            Some("block cannot be moved under its descendant".to_string())
        );
    });
}

#[test]
fn duplicate_block_clones_nested_subtree_after_source() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![
                    block(BLOCK_B, "paragraph", paragraph_payload("Parent")),
                    block(BLOCK_C, "paragraph", paragraph_payload("After source")),
                ],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_D, "to_do", todo_payload("Child", true))],
            },
        )
        .await
        .unwrap();

        writes::duplicate_block(
            &pool,
            BLOCK_B,
            NoteDuplicateBlock {
                duplicated_block_ids: vec![
                    NoteDuplicatedBlockId {
                        source_id: BLOCK_B.to_string(),
                        duplicate_id: BLOCK_E.to_string(),
                    },
                    NoteDuplicatedBlockId {
                        source_id: BLOCK_D.to_string(),
                        duplicate_id: BLOCK_F.to_string(),
                    },
                ],
            },
        )
        .await
        .unwrap();

        let duplicate_root = sqlx::query(
            "SELECT parent_type, parent_page_id, has_children, type, payload, sort_order
             FROM notes_blocks
             WHERE id = ?",
        )
        .bind(BLOCK_E)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(duplicate_root.get::<String, _>("parent_type"), "page_id");
        assert_eq!(duplicate_root.get::<String, _>("parent_page_id"), PAGE_A);
        assert_eq!(duplicate_root.get::<i64, _>("has_children"), 1);
        assert_eq!(duplicate_root.get::<String, _>("type"), "paragraph");
        let duplicate_payload: String = duplicate_root.get("payload");
        assert!(duplicate_payload.contains("Parent"));

        let duplicate_child = sqlx::query(
            "SELECT parent_type, parent_block_id, type, payload
             FROM notes_blocks
             WHERE id = ?",
        )
        .bind(BLOCK_F)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(duplicate_child.get::<String, _>("parent_type"), "block_id");
        assert_eq!(duplicate_child.get::<String, _>("parent_block_id"), BLOCK_E);
        assert_eq!(duplicate_child.get::<String, _>("type"), "to_do");
        let child_payload: String = duplicate_child.get("payload");
        assert!(child_payload.contains("\"checked\":true"));

        let source_order: f64 =
            sqlx::query_scalar("SELECT sort_order FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_B)
                .fetch_one(&pool)
                .await
                .unwrap();
        let duplicate_order: f64 = duplicate_root.get("sort_order");
        let next_order: f64 =
            sqlx::query_scalar("SELECT sort_order FROM notes_blocks WHERE id = ?")
                .bind(BLOCK_C)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(source_order < duplicate_order);
        assert!(duplicate_order < next_order);
    });
}

#[test]
fn duplicate_block_rejects_partial_subtree_id_maps() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Parent"))],
            },
        )
        .await
        .unwrap();
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: block_parent(BLOCK_B),
                after: None,
                children: vec![block(BLOCK_D, "paragraph", paragraph_payload("Child"))],
            },
        )
        .await
        .unwrap();

        let result = writes::duplicate_block(
            &pool,
            BLOCK_B,
            NoteDuplicateBlock {
                duplicated_block_ids: vec![NoteDuplicatedBlockId {
                    source_id: BLOCK_B.to_string(),
                    duplicate_id: BLOCK_E.to_string(),
                }],
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("duplicated_block_ids must match the source block subtree".to_string())
        );
    });
}

#[test]
fn schema_rejects_invalid_notes_rows() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        assert!(sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                type,
                payload,
                sort_order
             )
             VALUES (?, ?, 'page_id', ?, 'missing', '{}', 10)",
        )
        .bind(BLOCK_B)
        .bind(PAGE_A)
        .bind(PAGE_A)
        .execute(&pool)
        .await
        .is_err());

        assert!(
            sqlx::query("UPDATE notes_pages SET archived = 2 WHERE id = ?")
                .bind(PAGE_A)
                .execute(&pool)
                .await
                .is_err()
        );

        assert!(sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                type,
                payload,
                sort_order
             )
             VALUES (?, ?, 'page_id', ?, 'paragraph', 'not-json', 10)",
        )
        .bind(PAGE_B)
        .bind(PAGE_A)
        .bind(PAGE_A)
        .execute(&pool)
        .await
        .is_err());
    });
}

#[test]
fn local_user_identity_drives_notes_comments() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let local_user_json =
            serde_json::to_value(local_user::get_local_user(&pool).await.unwrap()).unwrap();
        let local_user_id = local_user_json["id"].as_str().unwrap().to_string();
        assert_ne!(local_user_id, "local-user");
        assert_eq!(local_user_json["display_name"], "You");

        let thread = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_A.to_string(),
                parent: Some(page_parent(PAGE_A)),
                discussion_id: None,
                anchor: None,
                rich_text: vec![rich_text("Page note")],
            },
        )
        .await
        .unwrap();
        let thread_value = serde_json::to_value(thread).unwrap();
        let thread_id = thread_value["id"].as_str().unwrap().to_string();
        assert_eq!(
            thread_value["comments"][0]["created_by"]["id"],
            local_user_id
        );
        assert_eq!(
            thread_value["comments"][0]["display_name"]["resolved_name"],
            "You"
        );

        let updated_user = local_user::update_local_user(
            &pool,
            NoteLocalUserUpdate {
                display_name: "Victor".to_string(),
            },
        )
        .await
        .unwrap();
        let updated_user_json = serde_json::to_value(updated_user).unwrap();
        assert_eq!(updated_user_json["id"], local_user_id);
        assert_eq!(updated_user_json["display_name"], "Victor");

        let listed = comments::list_comments(&pool, PAGE_A, false).await.unwrap();
        let listed_json = serde_json::to_value(listed).unwrap();
        assert_eq!(
            listed_json[0]["comments"][0]["display_name"]["resolved_name"],
            "Victor"
        );

        let replied = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_B.to_string(),
                parent: None,
                discussion_id: Some(thread_id.clone()),
                anchor: None,
                rich_text: vec![rich_text("Reply")],
            },
        )
        .await
        .unwrap();
        let replied_value = serde_json::to_value(replied).unwrap();
        assert_eq!(
            replied_value["comments"][1]["display_name"]["resolved_name"],
            "Victor"
        );

        let resolved = comments::resolve_comment_thread(&pool, &thread_id, true)
            .await
            .unwrap();
        let resolved_value = serde_json::to_value(resolved).unwrap();
        assert_eq!(resolved_value["resolved_by"]["id"], local_user_id);

        assert_eq!(
            local_user::update_local_user(
                &pool,
                NoteLocalUserUpdate {
                    display_name: " ".to_string(),
                },
            )
            .await
            .err()
            .as_deref(),
            Some("display_name is required")
        );
    });
}

#[test]
fn mention_notifications_sync_blocks_comments_and_delivery_state() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let local_user_json =
            serde_json::to_value(local_user::get_local_user(&pool).await.unwrap()).unwrap();
        let local_user_id = local_user_json["id"].as_str().unwrap().to_string();

        writes::update_block(
            &pool,
            BLOCK_A,
            block_update(
                "paragraph",
                json!({
                    "rich_text": [
                        rich_text("Plan "),
                        date_mention("2026-07-05", "Sunday", true),
                        rich_text(" with "),
                        user_mention(&local_user_id, "Victor"),
                        rich_text(" on "),
                        project_task_mention(BLOCK_B, "Task")
                    ],
                    "color": "default"
                }),
            ),
        )
        .await
        .unwrap();
        comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_A.to_string(),
                parent: Some(page_parent(PAGE_A)),
                discussion_id: None,
                anchor: None,
                rich_text: vec![rich_text("Ping "), user_mention(&local_user_id, "Victor")],
            },
        )
        .await
        .unwrap();

        let pending = mention_notifications::list_pending(&pool).await.unwrap();
        let pending_json = serde_json::to_value(&pending).unwrap();
        let mut kinds = pending_json
            .as_array()
            .unwrap()
            .iter()
            .map(|notification| notification["kind"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        kinds.sort();
        assert_eq!(
            kinds,
            vec![
                "reminder".to_string(),
                "task_mention".to_string(),
                "user_mention".to_string(),
                "user_mention".to_string(),
            ]
        );
        assert!(pending_json.as_array().unwrap().iter().any(|notification| {
            notification["source_type"] == "comment"
                && notification["comment_id"] == COMMENT_A
                && notification["page_title"] == "First page"
        }));

        let reminder_id = pending_json
            .as_array()
            .unwrap()
            .iter()
            .find(|notification| notification["kind"] == "reminder")
            .and_then(|notification| notification["id"].as_str())
            .unwrap()
            .to_string();
        let after_delivery = mention_notifications::mark_delivered(
            &pool,
            NoteMentionNotificationDeliveryUpdate {
                ids: vec![reminder_id.clone()],
            },
        )
        .await
        .unwrap();
        let after_delivery_json = serde_json::to_value(&after_delivery).unwrap();
        assert!(!after_delivery_json
            .as_array()
            .unwrap()
            .iter()
            .any(|notification| notification["id"] == reminder_id));

        writes::update_block(
            &pool,
            BLOCK_A,
            block_update("paragraph", paragraph_payload("No mentions")),
        )
        .await
        .unwrap();
        let after_block_clear =
            serde_json::to_value(mention_notifications::list_pending(&pool).await.unwrap())
                .unwrap();
        assert_eq!(after_block_clear.as_array().unwrap().len(), 1);
        assert_eq!(after_block_clear[0]["source_type"], "comment");
    });
}

#[test]
fn comments_create_reply_resolve_reopen_and_delete() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let thread = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_A.to_string(),
                parent: Some(page_parent(PAGE_A)),
                discussion_id: None,
                anchor: None,
                rich_text: vec![rich_text("Page note")],
            },
        )
        .await
        .unwrap();
        let thread_value = serde_json::to_value(thread).unwrap();
        let thread_id = thread_value["id"].as_str().unwrap().to_string();
        assert_eq!(thread_value["parent"]["type"], "page_id");
        assert_eq!(
            thread_value["comments"][0]["rich_text"][0]["plain_text"],
            "Page note"
        );

        let replied = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_B.to_string(),
                parent: None,
                discussion_id: Some(thread_id.clone()),
                anchor: None,
                rich_text: vec![rich_text("Reply")],
            },
        )
        .await
        .unwrap();
        let replied_value = serde_json::to_value(replied).unwrap();
        assert_eq!(replied_value["comments"].as_array().unwrap().len(), 2);

        let updated = comments::update_comment(
            &pool,
            COMMENT_B,
            NoteCommentUpdate {
                rich_text: vec![rich_text("Edited reply")],
            },
        )
        .await
        .unwrap();
        let updated_value = serde_json::to_value(updated).unwrap();
        assert_eq!(
            updated_value["comments"][1]["rich_text"][0]["plain_text"],
            "Edited reply"
        );

        let resolved = comments::resolve_comment_thread(&pool, &thread_id, true)
            .await
            .unwrap();
        let resolved_value = serde_json::to_value(resolved).unwrap();
        assert_eq!(resolved_value["status"], "resolved");
        assert!(resolved_value["resolved_at"].is_string());

        assert!(comments::list_comments(&pool, PAGE_A, false)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            comments::list_comments(&pool, PAGE_A, true)
                .await
                .unwrap()
                .len(),
            1
        );

        let reopened = comments::resolve_comment_thread(&pool, &thread_id, false)
            .await
            .unwrap();
        let reopened_value = serde_json::to_value(reopened).unwrap();
        assert_eq!(reopened_value["status"], "open");
        assert!(reopened_value["resolved_at"].is_null());

        let after_delete = comments::delete_comment(&pool, COMMENT_A).await.unwrap();
        let after_delete_value = serde_json::to_value(after_delete).unwrap();
        assert_eq!(after_delete_value["comments"].as_array().unwrap().len(), 1);
        assert_eq!(
            after_delete_value["comments"][0]["rich_text"][0]["plain_text"],
            "Edited reply"
        );
    });
}

#[test]
fn comment_unread_state_tracks_local_identity() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let local_user_json =
            serde_json::to_value(local_user::get_local_user(&pool).await.unwrap()).unwrap();
        let local_user_id = local_user_json["id"].as_str().unwrap().to_string();
        let thread = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_A.to_string(),
                parent: Some(page_parent(PAGE_A)),
                discussion_id: None,
                anchor: None,
                rich_text: vec![rich_text("Remote note")],
            },
        )
        .await
        .unwrap();
        let thread_value = serde_json::to_value(thread).unwrap();
        let thread_id = thread_value["id"].as_str().unwrap().to_string();
        assert_eq!(thread_value["unread"], false);

        sqlx::query("DELETE FROM notes_comment_thread_reads WHERE thread_id = ? AND user_id = ?")
            .bind(&thread_id)
            .bind(&local_user_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE notes_comments
             SET created_by = ?,
                 display_name = ?,
                 created_time = '2000-01-01T00:00:00.000Z',
                 last_edited_time = '2000-01-01T00:00:00.000Z'
             WHERE id = ?",
        )
        .bind("99999999-9999-4999-8999-999999999999")
        .bind(json!({"type": "user", "resolved_name": "Reviewer"}).to_string())
        .bind(COMMENT_A)
        .execute(&pool)
        .await
        .unwrap();

        let unread = comments::list_comments(&pool, PAGE_A, false).await.unwrap();
        let unread_value = serde_json::to_value(unread).unwrap();
        assert_eq!(unread_value[0]["unread"], true);

        let marked = comments::mark_comment_threads_read(
            &pool,
            NoteCommentThreadReadUpdate {
                page_id: PAGE_A.to_string(),
                discussion_ids: vec![thread_id.clone()],
                include_resolved: Some(false),
            },
        )
        .await
        .unwrap();
        let marked_value = serde_json::to_value(marked).unwrap();
        assert_eq!(marked_value[0]["unread"], false);

        let replied = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_B.to_string(),
                parent: None,
                discussion_id: Some(thread_id.clone()),
                anchor: None,
                rich_text: vec![rich_text("Local follow-up")],
            },
        )
        .await
        .unwrap();
        let replied_value = serde_json::to_value(replied).unwrap();
        assert_eq!(replied_value["unread"], false);

        sqlx::query(
            "UPDATE notes_comments
             SET last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '+1 second')
             WHERE id = ?",
        )
        .bind(COMMENT_A)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "UPDATE notes_comment_threads
             SET last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '+1 second')
             WHERE id = ?",
        )
        .bind(&thread_id)
        .execute(&pool)
        .await
        .unwrap();

        let edited = comments::list_comments(&pool, PAGE_A, false).await.unwrap();
        let edited_value = serde_json::to_value(edited).unwrap();
        assert_eq!(edited_value[0]["unread"], true);
    });
}

#[test]
fn block_comments_attach_to_visible_blocks() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(BLOCK_B, "paragraph", paragraph_payload("Anchored"))],
            },
        )
        .await
        .unwrap();

        let thread = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_C.to_string(),
                parent: Some(block_parent(BLOCK_B)),
                discussion_id: None,
                anchor: None,
                rich_text: vec![rich_text("Block note")],
            },
        )
        .await
        .unwrap();
        let thread_value = serde_json::to_value(thread).unwrap();
        assert_eq!(thread_value["parent"]["type"], "block_id");
        assert_eq!(thread_value["block_id"], BLOCK_B);

        let threads = comments::list_comments(&pool, PAGE_A, false).await.unwrap();
        assert_eq!(threads.len(), 1);

        writes::trash_block(&pool, BLOCK_B, true).await.unwrap();
        assert!(comments::list_comments(&pool, PAGE_A, false)
            .await
            .unwrap()
            .is_empty());
    });
}

#[test]
fn inline_comment_anchors_persist_on_block_threads() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "paragraph",
                    paragraph_payload("Alpha beta gamma"),
                )],
            },
        )
        .await
        .unwrap();

        let thread = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_A.to_string(),
                parent: Some(block_parent(BLOCK_B)),
                discussion_id: None,
                anchor: Some(NoteCommentAnchorCreate {
                    start: 6,
                    end: 10,
                    text: "beta".to_string(),
                    prefix: "Alpha ".to_string(),
                    suffix: " gamma".to_string(),
                }),
                rich_text: vec![rich_text("Inline note")],
            },
        )
        .await
        .unwrap();
        let thread_value = serde_json::to_value(thread).unwrap();
        let thread_id = thread_value["id"].as_str().unwrap();
        assert_eq!(thread_value["parent"]["type"], "block_id");
        assert_eq!(thread_value["anchor"]["type"], "text_range");
        assert_eq!(thread_value["anchor"]["block_id"], BLOCK_B);
        assert_eq!(thread_value["anchor"]["start"], 6);
        assert_eq!(thread_value["anchor"]["end"], 10);
        assert_eq!(thread_value["anchor"]["text"], "beta");

        let listed = comments::list_comments(&pool, PAGE_A, false).await.unwrap();
        let listed_value = serde_json::to_value(listed).unwrap();
        assert_eq!(listed_value[0]["anchor"]["text"], "beta");

        let reply_with_anchor = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_B.to_string(),
                parent: None,
                discussion_id: Some(thread_id.to_string()),
                anchor: Some(NoteCommentAnchorCreate {
                    start: 0,
                    end: 4,
                    text: "beta".to_string(),
                    prefix: String::new(),
                    suffix: String::new(),
                }),
                rich_text: vec![rich_text("Reply")],
            },
        )
        .await;
        assert_eq!(
            reply_with_anchor.err().as_deref(),
            Some("inline comment anchors can only start new block comment threads")
        );
    });
}

#[test]
fn suggestions_preserve_range_content_and_decision_state() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "paragraph",
                    paragraph_payload("Alpha beta gamma"),
                )],
            },
        )
        .await
        .unwrap();

        let suggestion = suggestions::create_suggestion(
            &pool,
            NoteSuggestionCreate {
                id: SUGGESTION_A.to_string(),
                block_id: BLOCK_B.to_string(),
                range_start: 6,
                range_end: 10,
                original_text: "beta".to_string(),
                proposed_text: "delta".to_string(),
                prefix: "Alpha ".to_string(),
                suffix: " gamma".to_string(),
            },
        )
        .await
        .unwrap();
        let suggestion_value = serde_json::to_value(suggestion).unwrap();
        assert_eq!(suggestion_value["object"], "suggestion");
        assert_eq!(suggestion_value["status"], "open");
        assert_eq!(suggestion_value["page_id"], PAGE_A);
        assert_eq!(suggestion_value["block_id"], BLOCK_B);
        assert_eq!(suggestion_value["range_start"], 6);
        assert_eq!(suggestion_value["range_end"], 10);
        assert_eq!(suggestion_value["original_text"], "beta");
        assert_eq!(suggestion_value["proposed_text"], "delta");
        assert_eq!(suggestion_value["display_name"]["resolved_name"], "You");
        assert!(suggestion_value["created_time"].is_string());

        let listed = suggestions::list_suggestions(&pool, PAGE_A, false)
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);

        let rejected = suggestions::reject_suggestion(&pool, SUGGESTION_A)
            .await
            .unwrap();
        let rejected_value = serde_json::to_value(rejected).unwrap();
        assert_eq!(rejected_value["status"], "rejected");
        assert!(rejected_value["rejected_at"].is_string());
        assert!(rejected_value["rejected_by"]["id"].is_string());
        assert!(suggestions::list_suggestions(&pool, PAGE_A, false)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            suggestions::list_suggestions(&pool, PAGE_A, true)
                .await
                .unwrap()
                .len(),
            1
        );

        let accepted = suggestions::create_suggestion(
            &pool,
            NoteSuggestionCreate {
                id: SUGGESTION_B.to_string(),
                block_id: BLOCK_B.to_string(),
                range_start: 11,
                range_end: 16,
                original_text: "gamma".to_string(),
                proposed_text: "omega".to_string(),
                prefix: " beta ".to_string(),
                suffix: String::new(),
            },
        )
        .await
        .unwrap();
        assert_eq!(serde_json::to_value(accepted).unwrap()["status"], "open");
        let accepted = suggestions::accept_suggestion(&pool, SUGGESTION_B)
            .await
            .unwrap();
        let accepted_value = serde_json::to_value(accepted).unwrap();
        assert_eq!(accepted_value["status"], "accepted");
        assert!(accepted_value["accepted_at"].is_string());
        assert!(accepted_value["accepted_by"]["id"].is_string());

        let missing_original = suggestions::create_suggestion(
            &pool,
            NoteSuggestionCreate {
                id: "60606060-6060-4060-8060-606060606060".to_string(),
                block_id: BLOCK_B.to_string(),
                range_start: 0,
                range_end: 7,
                original_text: "missing".to_string(),
                proposed_text: "present".to_string(),
                prefix: String::new(),
                suffix: String::new(),
            },
        )
        .await;
        assert_eq!(
            missing_original.err().as_deref(),
            Some("suggestion original text must exist in the block")
        );
    });
}

#[test]
fn collaboration_operations_track_comments_and_suggestions_for_future_sync() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        writes::append_block_children(
            &pool,
            NoteAppendBlockChildren {
                parent: page_parent(PAGE_A),
                after: Some(BLOCK_A.to_string()),
                children: vec![block(
                    BLOCK_B,
                    "paragraph",
                    paragraph_payload("Alpha beta gamma"),
                )],
            },
        )
        .await
        .unwrap();

        let thread = comments::create_comment(
            &pool,
            NoteCommentCreate {
                id: COMMENT_A.to_string(),
                parent: Some(block_parent(BLOCK_B)),
                discussion_id: None,
                anchor: Some(NoteCommentAnchorCreate {
                    start: 6,
                    end: 10,
                    text: "beta".to_string(),
                    prefix: "Alpha ".to_string(),
                    suffix: " gamma".to_string(),
                }),
                rich_text: vec![rich_text("First note")],
            },
        )
        .await
        .unwrap();
        let thread_value = serde_json::to_value(thread).unwrap();
        let thread_id = thread_value["id"].as_str().unwrap().to_string();

        comments::update_comment(
            &pool,
            COMMENT_A,
            NoteCommentUpdate {
                rich_text: vec![rich_text("Edited note")],
            },
        )
        .await
        .unwrap();
        comments::resolve_comment_thread(&pool, &thread_id, true)
            .await
            .unwrap();
        comments::resolve_comment_thread(&pool, &thread_id, false)
            .await
            .unwrap();
        comments::delete_comment(&pool, COMMENT_A).await.unwrap();

        suggestions::create_suggestion(
            &pool,
            NoteSuggestionCreate {
                id: SUGGESTION_A.to_string(),
                block_id: BLOCK_B.to_string(),
                range_start: 6,
                range_end: 10,
                original_text: "beta".to_string(),
                proposed_text: "delta".to_string(),
                prefix: "Alpha ".to_string(),
                suffix: " gamma".to_string(),
            },
        )
        .await
        .unwrap();
        suggestions::reject_suggestion(&pool, SUGGESTION_A)
            .await
            .unwrap();

        let rows = sqlx::query(
            "SELECT
                entity_type,
                entity_id,
                operation_type,
                base_version,
                entity_version,
                conflict_policy,
                actor_display_name,
                payload
             FROM notes_collaboration_operations
             WHERE page_id = ?
             ORDER BY sequence ASC",
        )
        .bind(PAGE_A)
        .fetch_all(&pool)
        .await
        .unwrap();
        let operation_types: Vec<String> = rows
            .iter()
            .map(|row| row.try_get::<String, _>("operation_type").unwrap())
            .collect();
        assert_eq!(
            operation_types,
            vec![
                "comment_thread_create",
                "comment_create",
                "comment_update",
                "comment_thread_resolve",
                "comment_thread_reopen",
                "comment_delete",
                "suggestion_create",
                "suggestion_reject",
            ]
        );

        let versions: Vec<(i64, i64, String)> = rows
            .iter()
            .map(|row| {
                (
                    row.try_get::<i64, _>("base_version").unwrap(),
                    row.try_get::<i64, _>("entity_version").unwrap(),
                    row.try_get::<String, _>("conflict_policy").unwrap(),
                )
            })
            .collect();
        assert_eq!(
            versions,
            vec![
                (0, 1, "append_only".to_string()),
                (0, 1, "append_only".to_string()),
                (1, 2, "last_writer_wins".to_string()),
                (1, 2, "state_transition".to_string()),
                (2, 3, "state_transition".to_string()),
                (2, 3, "state_transition".to_string()),
                (0, 1, "append_only".to_string()),
                (1, 2, "state_transition".to_string()),
            ]
        );

        let thread_payload: serde_json::Value =
            serde_json::from_str(&rows[0].try_get::<String, _>("payload").unwrap()).unwrap();
        assert_eq!(thread_payload["anchor"]["text"], "beta");
        assert_eq!(thread_payload["parent"]["block_id"], BLOCK_B);

        let update_payload: serde_json::Value =
            serde_json::from_str(&rows[2].try_get::<String, _>("payload").unwrap()).unwrap();
        assert_eq!(update_payload["rich_text"][0]["plain_text"], "Edited note");

        let reject_payload: serde_json::Value =
            serde_json::from_str(&rows[7].try_get::<String, _>("payload").unwrap()).unwrap();
        assert_eq!(reject_payload["status"], "rejected");
        assert_eq!(reject_payload["original_text"], "beta");
        assert_eq!(reject_payload["proposed_text"], "delta");

        for row in rows {
            let display_name: serde_json::Value =
                serde_json::from_str(&row.try_get::<String, _>("actor_display_name").unwrap())
                    .unwrap();
            assert_eq!(display_name["resolved_name"], "You");
        }
    });
}
