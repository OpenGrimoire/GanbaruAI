use super::models::{
    NoteAppendBlockChildren, NoteBlockUpdate, NoteBlockWrite, NoteChildPageFromBlockCreate,
    NoteCommentCreate, NoteCommentUpdate, NoteDuplicateBlock, NoteDuplicateBlocks,
    NoteDuplicatePage, NoteDuplicatedBlockId, NoteMoveBlock, NoteMoveBlocks, NoteMovePage,
    NotePageCreate, NoteParent, NoteTrashBlocks, OptionalJsonValue,
};
use super::{comments, reads, undo_state, validation, writes};
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
        Err("rich text mention.type must be page or date".to_string())
    );
    assert_eq!(
        validation::validate_block_payload("nope", &json!({})),
        Err("unsupported block type: nope".to_string())
    );
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
    assert!(
        validation::validate_block_payload("child_database", &child_database_payload("Tasks"),)
            .is_ok()
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
fn page_cover_validation_rejects_non_image_urls() {
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
                icon: OptionalJsonValue::Unset,
                cover: OptionalJsonValue::Value(json!({
                    "type": "external",
                    "external": {
                        "url": "https://example.com/document.pdf"
                    }
                })),
            },
        )
        .await;

        assert_eq!(
            result.err(),
            Some("cover.external.url must be a supported HTTPS image URL".to_string())
        );
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

        writes::update_block(&pool, BLOCK_B, block_update("tab", tab_payload()))
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
            "Overview"
        );
        assert_eq!(
            tab_children_json["results"][0]["paragraph"]["icon"]["icon"]["name"],
            "star"
        );

        let first_tab_panel = reads::get_block_children(&pool, BLOCK_C, None, Some(10))
            .await
            .unwrap();
        let first_tab_panel_json = serde_json::to_value(first_tab_panel).unwrap();
        assert_eq!(first_tab_panel_json["results"][0]["type"], "to_do");
        assert_eq!(
            first_tab_panel_json["results"][0]["to_do"]["rich_text"][0]["plain_text"],
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
            0.5
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
        assert_eq!(plain_text, "button Action content is not exposed");
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
