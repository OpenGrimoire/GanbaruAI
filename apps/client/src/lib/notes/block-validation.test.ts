import { describe, expect, it } from "vitest";
import {
  parseNotesBlock,
  parseNotesCreatedDatabase,
  parseNotesDataSourceCsvExportSaveResult,
  parseNotesDataSourceCsvImportResult,
  parseNotesHtmlArchiveSaveResult,
  parseNotesHtmlExportResult,
  parseNotesLocalUser,
  parseNotesMentionNotification,
  parseNotesHtmlImportResult,
  parseNotesMarkdownExportResult,
  parseNotesNotionApiImportResult,
  parseNotesPage,
  parseNotesPageHistorySettings,
  parseNotesPageHistorySnapshot,
  parseNotesPageTemplate,
  parseNotesRichTextArray,
  parseNotesSearchResult,
  parseNotesPageAlias,
  parseNotesUnresolvedLink,
} from "./block-validation";

const basePage = {
  object: "page",
  id: "11111111-1111-4111-8111-111111111111",
  created_time: "2026-06-30T12:00:00.000Z",
  last_edited_time: "2026-06-30T12:00:00.000Z",
  parent: { type: "workspace", workspace: true },
  in_trash: false,
  archived: false,
  icon: null,
  cover: null,
  properties: {},
  url: null,
  public_url: null,
  source_provider: null,
  source_object_id: null,
  source_workspace_id: null,
  source_last_edited_time: null,
};

const baseBlock = {
  object: "block",
  id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  parent: { type: "page_id", page_id: basePage.id },
  created_time: "2026-06-30T12:00:00.000Z",
  last_edited_time: "2026-06-30T12:00:00.000Z",
  has_children: false,
  in_trash: false,
  archived: false,
  source_provider: null,
  source_object_id: null,
  source_last_edited_time: null,
};

const baseRichText = {
  type: "text",
  text: {
    content: "Heading",
    link: null,
  },
  annotations: {
    bold: false,
    italic: false,
    strikethrough: false,
    underline: false,
    code: false,
    color: "default",
  },
  plain_text: "Heading",
  href: null,
};

describe("notes boundary validation", () => {
  it("parses page emoji icons", () => {
    const page = parseNotesPage({
      ...basePage,
      icon: { type: "emoji", emoji: "📌" },
    });

    expect(page.icon).toEqual({ type: "emoji", emoji: "📌" });
  });

  it("parses data source row page parents", () => {
    const page = parseNotesPage({
      ...basePage,
      parent: {
        type: "data_source_id",
        data_source_id: "81818181-8181-4181-8181-818181818181",
      },
    });

    expect(page.parent).toEqual({
      type: "data_source_id",
      data_source_id: "81818181-8181-4181-8181-818181818181",
    });
  });

  it("rejects empty page emoji icons", () => {
    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: { type: "emoji", emoji: "" },
      }),
    ).toThrow("page.icon.emoji must not be empty");
  });

  it("parses expanded page icon payloads", () => {
    expect(parseNotesPage({
      ...basePage,
      icon: { type: "icon", icon: { name: "home", color: "blue" } },
    }).icon).toEqual({ type: "icon", icon: { name: "home", color: "blue" } });

    expect(parseNotesPage({
      ...basePage,
      icon: {
        type: "custom_emoji",
        custom_emoji: {
          id: "emoji-a",
          name: "Focus",
          url: "ganbaru-asset:project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
          ganbaru_asset_path: "project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        },
      },
    }).icon).toEqual({
      type: "custom_emoji",
      custom_emoji: {
        id: "emoji-a",
        name: "Focus",
        url: "ganbaru-asset:project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        ganbaru_asset_path: "project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
      },
    });

    expect(parseNotesPage({
      ...basePage,
      icon: { type: "external", external: { url: "https://example.com/icon.png" } },
    }).icon).toEqual({ type: "external", external: { url: "https://example.com/icon.png" } });

    expect(parseNotesPage({
      ...basePage,
      icon: {
        type: "file",
        file: {
          url: "ganbaru-asset:notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.webp",
          name: "Focus",
          content_type: "image/webp",
          byte_size: 42,
          sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
          ganbaru_asset_path: "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.webp",
        },
      },
    }).icon).toEqual({
      type: "file",
      file: {
        url: "ganbaru-asset:notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.webp",
        name: "Focus",
        content_type: "image/webp",
        byte_size: 42,
        sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ganbaru_asset_path: "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.webp",
      },
    });
  });

  it("parses mention notification DTOs", () => {
    expect(parseNotesMentionNotification({
      object: "mention_notification",
      id: "notification-a",
      source_type: "comment",
      source_id: "10101010-1010-4010-8010-101010101010",
      page_id: basePage.id,
      page_title: "Inbox",
      block_id: baseBlock.id,
      comment_id: "10101010-1010-4010-8010-101010101010",
      kind: "user_mention",
      target_type: "user",
      target_id: "12121212-1212-4212-8212-121212121212",
      trigger_at: null,
      plain_text: "Victor",
      source_plain_text: "Hi Victor",
      status: "pending",
      delivered_at: null,
      created_time: "2026-07-01T12:00:00.000Z",
      last_edited_time: "2026-07-01T12:00:00.000Z",
    })).toMatchObject({
      source_type: "comment",
      kind: "user_mention",
      target_type: "user",
      status: "pending",
    });
  });

  it("rejects unsafe expanded page icon payloads", () => {
    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: { type: "external", external: { url: "http://example.com/icon.png" } },
      }),
    ).toThrow("page.icon.external.url must be a supported HTTPS image URL");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: { type: "custom_emoji", custom_emoji: { id: "", name: "Missing id" } },
      }),
    ).toThrow("page.icon.custom_emoji.id must not be empty");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: {
          type: "custom_emoji",
          custom_emoji: {
            id: "wrong-directory",
            url: "ganbaru-asset:notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
          },
        },
      }),
    ).toThrow("page.icon.custom_emoji.url must stay under a managed image asset directory");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: {
          type: "custom_emoji",
          custom_emoji: {
            id: "missing-asset-path",
            url: "ganbaru-asset:project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
          },
        },
      }),
    ).toThrow("page.icon.custom_emoji.url must reference the managed icon asset path");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: {
          type: "file",
          file: {
            url: "ganbaru-asset:notes/page-icons/bad.svg",
            content_type: "image/svg+xml",
            byte_size: 42,
            sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ganbaru_asset_path: "notes/page-icons/bad.svg",
          },
        },
      }),
    ).toThrow("page.icon.file.ganbaru_asset_path must stay under a managed image asset directory");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: {
          type: "file",
          file: {
            url: "ganbaru-asset:notes/page-icons/cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc.webp",
            content_type: "image/webp",
            byte_size: 42,
            sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ganbaru_asset_path: "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.webp",
          },
        },
      }),
    ).toThrow("page.icon.file.url must reference the managed icon asset path");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: {
          type: "file",
          file: {
            url: "ganbaru-asset:notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.webp",
          },
        },
      }),
    ).toThrow("page.icon.file.url must include managed asset metadata");
  });

  it("parses page template DTOs", () => {
    const template = parseNotesPageTemplate({
      object: "page_template",
      id: "99999999-9999-4999-8999-999999999999",
      name: "Weekly review",
      source_page_id: basePage.id,
      properties: {
        title: {
          id: "title",
          type: "title",
          title: [baseRichText],
        },
      },
      icon: { type: "emoji", emoji: "📄" },
      cover: null,
      block_count: 3,
      created_time: "2026-07-01T12:00:00.000Z",
      last_edited_time: "2026-07-01T12:00:00.000Z",
    });

    expect(template.name).toBe("Weekly review");
    expect(template.source_page_id).toBe(basePage.id);
    expect(template.block_count).toBe(3);
  });

  it("rejects negative page template block counts", () => {
    expect(() =>
      parseNotesPageTemplate({
        object: "page_template",
        id: "99999999-9999-4999-8999-999999999999",
        name: "Weekly review",
        source_page_id: null,
        properties: {},
        icon: null,
        cover: null,
        block_count: -1,
        created_time: "2026-07-01T12:00:00.000Z",
        last_edited_time: "2026-07-01T12:00:00.000Z",
      }),
    ).toThrow("page template.block_count must not be negative");
  });

  it("parses page history snapshot DTOs", () => {
    const snapshot = parseNotesPageHistorySnapshot({
      object: "page_history_snapshot",
      id: "88888888-8888-4888-8888-888888888888",
      page_id: basePage.id,
      title: "Draft",
      icon: { type: "emoji", emoji: "🕘" },
      cover: null,
      block_count: 4,
      reason: "update_block",
      created_by: { object: "user", id: "70707070-7070-4070-8070-707070707070" },
      created_time: "2026-07-01T12:00:00.000Z",
      page_last_edited_time: "2026-07-01T11:59:00.000Z",
    });

    expect(snapshot.page_id).toBe(basePage.id);
    expect(snapshot.block_count).toBe(4);
    expect(snapshot.reason).toBe("update_block");
    expect(snapshot.created_by.id).toBe("70707070-7070-4070-8070-707070707070");
  });

  it("rejects invalid page history snapshot block counts", () => {
    expect(() =>
      parseNotesPageHistorySnapshot({
        object: "page_history_snapshot",
        id: "88888888-8888-4888-8888-888888888888",
        page_id: basePage.id,
        title: "Draft",
        icon: null,
        cover: null,
        block_count: -1,
        reason: "update_block",
        created_by: { object: "user", id: "70707070-7070-4070-8070-707070707070" },
        created_time: "2026-07-01T12:00:00.000Z",
        page_last_edited_time: "2026-07-01T11:59:00.000Z",
      }),
    ).toThrow("page history snapshot.block_count must not be negative");
  });

  it("parses local Notes user DTOs", () => {
    const user = parseNotesLocalUser({
      object: "user",
      id: "70707070-7070-4070-8070-707070707070",
      display_name: "Victor",
      created_time: "2026-07-01T12:00:00.000Z",
      last_edited_time: "2026-07-01T12:01:00.000Z",
    });

    expect(user.display_name).toBe("Victor");
  });

  it("parses page history settings DTOs", () => {
    expect(
      parseNotesPageHistorySettings({
        object: "page_history_settings",
        retention_days: 90,
        updated_at: "2026-07-01T12:00:00.000Z",
      }),
    ).toMatchObject({ retention_days: 90 });

    expect(
      parseNotesPageHistorySettings({
        object: "page_history_settings",
        retention_days: null,
        updated_at: "2026-07-01T12:00:00.000Z",
      }),
    ).toMatchObject({ retention_days: null });
  });

  it("rejects invalid page history retention windows", () => {
    expect(() =>
      parseNotesPageHistorySettings({
        object: "page_history_settings",
        retention_days: 0,
        updated_at: "2026-07-01T12:00:00.000Z",
      }),
    ).toThrow("page history settings.retention_days must be between 1 and 3650");
  });

  it("parses toggleable heading payload state", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "heading_2",
      heading_2: {
        rich_text: [baseRichText],
        color: "default",
        is_toggleable: true,
        ganbaru_open: false,
      },
    });

    expect(block).toMatchObject({
      type: "heading_2",
      heading_2: {
        is_toggleable: true,
        ganbaru_open: false,
      },
    });
  });

  it("parses heading 4 payloads", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "heading_4",
      heading_4: {
        rich_text: [baseRichText],
        color: "blue",
        is_toggleable: true,
        ganbaru_open: true,
      },
    });

    expect(block.type).toBe("heading_4");
    if (block.type === "heading_4") {
      expect(block.heading_4.color).toBe("blue");
      expect(block.heading_4.is_toggleable).toBe(true);
      expect(block.heading_4.ganbaru_open).toBe(true);
    }
  });

  it("parses paragraph icons used by tab labels", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "paragraph",
      paragraph: {
        rich_text: [baseRichText],
        color: "default",
        icon: { type: "icon", icon: { name: "star", color: "yellow" } },
      },
    });

    expect(block.type).toBe("paragraph");
    if (block.type === "paragraph") {
      expect(block.paragraph.icon).toEqual({
        type: "icon",
        icon: { name: "star", color: "yellow" },
      });
    }
  });

  it("parses local managed media file objects", () => {
    const assetPath = "notes/files/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png";
    const block = parseNotesBlock({
      ...baseBlock,
      type: "image",
      image: {
        type: "file",
        file: {
          url: `ganbaru-asset:${assetPath}`,
          name: "local.png",
          content_type: "image/png",
          byte_size: 42,
          sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          ganbaru_asset_path: assetPath,
        },
        caption: [baseRichText],
        name: "local.png",
      },
    });

    expect(block.type).toBe("image");
    if (block.type !== "image" || block.image.type !== "file") {
      throw new Error("Expected local image file block");
    }
    expect(block.image.file).toEqual({
      url: `ganbaru-asset:${assetPath}`,
      name: "local.png",
      content_type: "image/png",
      byte_size: 42,
      sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      ganbaru_asset_path: assetPath,
    });
  });

  it("rejects unsafe local managed media file objects", () => {
    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "pdf",
        pdf: {
          type: "file",
          file: {
            url: "ganbaru-asset:notes/files/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.pdf",
            name: "brief.pdf",
            content_type: "text/plain",
            byte_size: 42,
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ganbaru_asset_path:
              "notes/files/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.pdf",
          },
          caption: [],
        },
      }),
    ).toThrow("block.pdf.file.content_type must match the local media block type");

    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "image",
        image: {
          type: "file",
          file: {
            url: "ganbaru-asset:notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
            content_type: "image/png",
            byte_size: 42,
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ganbaru_asset_path:
              "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
          },
          caption: [],
        },
      }),
    ).toThrow("block.image.file.ganbaru_asset_path must stay under the managed Notes file directory");

    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "image",
        image: {
          type: "file",
          file: {
            url: "ganbaru-asset:notes/files/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
          },
          caption: [],
        },
      }),
    ).toThrow("block.image.file.url must include managed asset metadata");
  });

  it("rejects invalid toggleable heading flags", () => {
    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "heading_1",
        heading_1: {
          rich_text: [baseRichText],
          color: "default",
          is_toggleable: "yes",
        },
      }),
    ).toThrow("block.heading_1.is_toggleable must be a boolean");
  });

  it("parses link preview payloads", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "link_preview",
      link_preview: {
        url: "https://github.com/example/repo/pull/123",
      },
    });

    expect(block.type).toBe("link_preview");
    if (block.type === "link_preview") {
      expect(block.link_preview.url).toBe("https://github.com/example/repo/pull/123");
    }
  });

  it("rejects link preview URLs with control characters", () => {
    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "link_preview",
        link_preview: {
          url: "https://example.com/bad\u0008",
        },
      }),
    ).toThrow("block.link_preview.url must not contain control characters");
  });

  it("parses child database payloads", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "child_database",
      child_database: {
        title: "Tasks",
        database_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        data_source_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
        view_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
      },
    });

    expect(block.type).toBe("child_database");
    if (block.type === "child_database") {
      expect(block.child_database.title).toBe("Tasks");
      expect(block.child_database.database_id).toBe("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb");
      expect(block.child_database.data_source_id).toBe("cccccccc-cccc-4ccc-8ccc-cccccccccccc");
      expect(block.child_database.view_id).toBe("dddddddd-dddd-4ddd-8ddd-dddddddddddd");
    }
  });

  it("parses created local database responses", () => {
    const created = parseNotesCreatedDatabase({
      database: {
        object: "database",
        id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        parent: baseBlock.parent,
        title: "Tasks",
        title_rich_text: [baseRichText],
        description: [],
        icon: null,
        cover: null,
        in_trash: false,
        is_inline: true,
        data_sources: [{ id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", name: "Tasks" }],
        url: null,
        public_url: null,
        source_provider: null,
        source_object_id: null,
        source_workspace_id: null,
        source_last_edited_time: null,
        created_time: baseBlock.created_time,
        last_edited_time: baseBlock.last_edited_time,
      },
      data_source: {
        object: "data_source",
        id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
        parent: {
          type: "database_id",
          database_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        },
        database_parent: baseBlock.parent,
        title: "Tasks",
        title_rich_text: [baseRichText],
        description: [],
        icon: null,
        properties: {
          Name: { id: "title", name: "Name", type: "title", title: {} },
        },
        in_trash: false,
        source_provider: null,
        source_object_id: null,
        source_workspace_id: null,
        source_last_edited_time: null,
        created_time: baseBlock.created_time,
        last_edited_time: baseBlock.last_edited_time,
      },
      view: {
        object: "view",
        id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
        parent: {
          type: "database_id",
          database_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        },
        data_source_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
        name: "Table",
        type: "table",
        filter: null,
        sorts: [],
        configuration: {
          type: "table",
          table: {
            property_order: ["title"],
            hidden_property_ids: [],
          },
        },
        url: null,
        source_provider: null,
        source_object_id: null,
        source_workspace_id: null,
        source_last_edited_time: null,
        created_time: baseBlock.created_time,
        last_edited_time: baseBlock.last_edited_time,
      },
      block: {
        ...baseBlock,
        type: "child_database",
        child_database: {
          title: "Tasks",
          database_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
          data_source_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
          view_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
        },
      },
    });

    expect(created.database.data_sources[0]?.id).toBe("cccccccc-cccc-4ccc-8ccc-cccccccccccc");
    expect(created.view.type).toBe("table");
    expect(created.block.child_database.database_id).toBe("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb");
  });

  it("rejects child database titles with control characters", () => {
    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "child_database",
        child_database: {
          title: "Tasks\u0008",
        },
      }),
    ).toThrow("block.child_database.title must not contain control characters");
  });

  it("parses tab block payloads", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "tab",
      tab: {},
    });

    expect(block.type).toBe("tab");
    if (block.type === "tab") {
      expect(block.tab).toEqual({});
    }
  });

  it("rejects tab block payload properties", () => {
    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "tab",
        tab: { title: "Bad" },
      }),
    ).toThrow("block.tab must be an empty object");
  });

  it("parses original synced block payloads", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "synced_block",
      synced_block: {
        synced_from: null,
      },
    });

    expect(block.type).toBe("synced_block");
    if (block.type === "synced_block") {
      expect(block.synced_block.synced_from).toBeNull();
    }
  });

  it("parses duplicate synced block references", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "synced_block",
      synced_block: {
        synced_from: {
          type: "block_id",
          block_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        },
      },
    });

    expect(block.type).toBe("synced_block");
    if (block.type === "synced_block") {
      expect(block.synced_block.synced_from?.block_id).toBe(
        "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
      );
    }
  });

  it("rejects invalid synced block references", () => {
    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "synced_block",
        synced_block: {
          synced_from: {
            type: "page_id",
            block_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
          },
        },
      }),
    ).toThrow("block.synced_block.synced_from.type must be block_id");

    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "synced_block",
        synced_block: {
          synced_from: {
            type: "block_id",
            block_id: "bad",
          },
        },
      }),
    ).toThrow("block.synced_block.synced_from.block_id must be a UUID");
  });

  it("parses template block payloads", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "template",
      template: {
        rich_text: [baseRichText],
      },
    });

    expect(block.type).toBe("template");
    if (block.type === "template") {
      expect(block.template.rich_text[0]?.plain_text).toBe("Heading");
    }
  });

  it("rejects invalid template rich text payloads", () => {
    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "template",
        template: {
          rich_text: "Add task",
        },
      }),
    ).toThrow("block.template.rich_text must be an array");

    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "template",
        template: {
          rich_text: [baseRichText],
          color: "default",
        },
      }),
    ).toThrow("block.template.color is not supported");

    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "template",
        template: {
          rich_text: [baseRichText],
          children: [],
        },
      }),
    ).toThrow("block.template.children must be stored as child blocks");
  });

  it("parses button block payloads", () => {
    const block = parseNotesBlock({
      ...baseBlock,
      type: "button",
      button: {
        rich_text: [baseRichText],
        icon: { type: "icon", icon: { name: "mouse-pointer-click", color: "gray" } },
        actions: [{ type: "insert_blocks", source: "children", position: "below_button" }],
      },
    });

    expect(block.type).toBe("button");
    if (block.type === "button") {
      expect(block.button.rich_text[0]?.plain_text).toBe("Heading");
      expect(block.button.actions[0]?.position).toBe("below_button");
    }
  });

  it("rejects invalid button action payloads", () => {
    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "button",
        button: {
          rich_text: [baseRichText],
          icon: null,
          actions: [],
        },
      }),
    ).toThrow("block.button.actions must include between 1 and 10 actions");

    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "button",
        button: {
          rich_text: [baseRichText],
          icon: null,
          actions: [{ type: "send_webhook", source: "children", position: "below_button" }],
        },
      }),
    ).toThrow("block.button.actions[0].type must be insert_blocks");

    expect(() =>
      parseNotesBlock({
        ...baseBlock,
        type: "button",
        button: {
          rich_text: [baseRichText],
          icon: null,
          children: [],
          actions: [{ type: "insert_blocks", source: "children", position: "below_button" }],
        },
      }),
    ).toThrow("block.button.children must be stored as child blocks");
  });

  it("parses page covers as image file objects", () => {
    const externalPage = parseNotesPage({
      ...basePage,
      icon: null,
      cover: { type: "external", external: { url: "https://example.com/cover.jpg" } },
    });

    expect(externalPage.cover).toEqual({
      type: "external",
      external: { url: "https://example.com/cover.jpg" },
    });

    const importedFilePage = parseNotesPage({
      ...basePage,
      icon: null,
      cover: {
        type: "file",
        file: {
          url: "https://example.com/imported-cover.webp",
          expiry_time: "2026-07-01T12:00:00.000Z",
        },
      },
    });
    expect(importedFilePage.cover).toEqual({
      type: "file",
      file: {
        url: "https://example.com/imported-cover.webp",
        expiry_time: "2026-07-01T12:00:00.000Z",
      },
    });

    const localFilePage = parseNotesPage({
      ...basePage,
      icon: null,
      cover: {
        type: "file",
        file: {
          url: "ganbaru-asset:notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
          name: "Cover",
          content_type: "image/png",
          byte_size: 42,
          sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          ganbaru_asset_path: "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        },
      },
    });
    expect(localFilePage.cover).toEqual({
      type: "file",
      file: {
        url: "ganbaru-asset:notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        name: "Cover",
        content_type: "image/png",
        byte_size: 42,
        sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ganbaru_asset_path: "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
      },
    });

    const fileUploadPage = parseNotesPage({
      ...basePage,
      icon: null,
      cover: {
        type: "file_upload",
        file_upload: { id: "55555555-5555-4555-8555-555555555555" },
      },
    });
    expect(fileUploadPage.cover).toEqual({
      type: "file_upload",
      file_upload: { id: "55555555-5555-4555-8555-555555555555" },
    });
  });

  it("rejects unsafe page cover file objects", () => {
    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: null,
        cover: { type: "external", external: { url: "https://example.com/file.pdf" } },
      }),
    ).toThrow("page.cover.external.url must be a supported HTTPS image URL");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: null,
        cover: {
          type: "file",
          file: {
            url: "https://example.com/file.pdf",
            expiry_time: "2026-07-01T12:00:00.000Z",
          },
        },
      }),
    ).toThrow("page.cover.file.url must be a supported HTTPS image URL");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: null,
        cover: {
          type: "file",
          file: {
            url: "ganbaru-asset:notes/page-covers/bad.svg",
            content_type: "image/svg+xml",
            byte_size: 42,
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ganbaru_asset_path: "notes/page-covers/bad.svg",
          },
        },
      }),
    ).toThrow("page.cover.file.ganbaru_asset_path must stay under a managed image asset directory");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: null,
        cover: {
          type: "file",
          file: {
            url: "ganbaru-asset:notes/page-covers/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
            content_type: "image/png",
            byte_size: 42,
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ganbaru_asset_path: "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
          },
        },
      }),
    ).toThrow("page.cover.file.url must reference the managed cover asset path");

    expect(() =>
      parseNotesPage({
        ...basePage,
        icon: null,
        cover: {
          type: "file",
          file: {
            url: "ganbaru-asset:notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
          },
        },
      }),
    ).toThrow("page.cover.file.url must include managed asset metadata");
  });

  it("parses page mention rich text", () => {
    const richText = parseNotesRichTextArray(
      [
        {
          type: "mention",
          mention: {
            type: "page",
            page: { id: "22222222-2222-4222-8222-222222222222" },
          },
          annotations: {
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            code: false,
            color: "default",
          },
          plain_text: "Target page",
          href: "http://localhost:1420/?view=notes#notes?page=22222222-2222-4222-8222-222222222222",
        },
      ],
      "rich_text",
    );

    expect(richText[0]).toMatchObject({
      type: "mention",
      mention: { type: "page", page: { id: "22222222-2222-4222-8222-222222222222" } },
    });
  });

  it("parses linked text rich text", () => {
    const richText = parseNotesRichTextArray(
      [
        {
          type: "text",
          text: {
            content: "docs",
            link: { url: "https://example.com/docs" },
          },
          annotations: {
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            code: false,
            color: "default",
          },
          plain_text: "docs",
          href: "https://example.com/docs",
        },
      ],
      "rich_text",
    );

    expect(richText[0]).toMatchObject({
      type: "text",
      text: { link: { url: "https://example.com/docs" } },
      href: "https://example.com/docs",
    });
  });

  it("parses email linked text rich text", () => {
    const richText = parseNotesRichTextArray(
      [
        {
          type: "text",
          text: {
            content: "email",
            link: { url: "mailto:team@example.com" },
          },
          annotations: {
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            code: false,
            color: "default",
          },
          plain_text: "email",
          href: "mailto:team@example.com",
        },
      ],
      "rich_text",
    );

    expect(richText[0]).toMatchObject({
      type: "text",
      text: { link: { url: "mailto:team@example.com" } },
      href: "mailto:team@example.com",
    });
  });

  it("parses annotated text rich text", () => {
    const richText = parseNotesRichTextArray(
      [
        {
          type: "text",
          text: {
            content: "important",
            link: null,
          },
          annotations: {
            bold: true,
            italic: true,
            strikethrough: false,
            underline: true,
            code: false,
            color: "blue_background",
          },
          plain_text: "important",
          href: null,
        },
      ],
      "rich_text",
    );

    expect(richText[0]).toMatchObject({
      type: "text",
      annotations: {
        bold: true,
        italic: true,
        underline: true,
        color: "blue_background",
      },
    });
  });

  it("parses inline equation rich text", () => {
    const richText = parseNotesRichTextArray(
      [
        {
          type: "equation",
          equation: {
            expression: "\\frac{a}{b}",
          },
          annotations: {
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            code: false,
            color: "default",
          },
          plain_text: "\\frac{a}{b}",
          href: null,
        },
      ],
      "rich_text",
    );

    expect(richText[0]).toMatchObject({
      type: "equation",
      equation: { expression: "\\frac{a}{b}" },
      plain_text: "\\frac{a}{b}",
    });
  });

  it("rejects invalid inline equation rich text", () => {
    expect(() =>
      parseNotesRichTextArray(
        [
          {
            type: "equation",
            equation: {
              expression: "bad\u0008",
            },
            annotations: {
              bold: false,
              italic: false,
              strikethrough: false,
              underline: false,
              code: false,
              color: "default",
            },
            plain_text: "bad",
            href: null,
          },
        ],
        "rich_text",
      ),
    ).toThrow("rich_text[0].equation.expression must not be empty, too long, or contain control characters");
  });

  it("rejects unsupported annotation colors", () => {
    expect(() =>
      parseNotesRichTextArray(
        [
          {
            type: "text",
            text: {
              content: "bad",
              link: null,
            },
            annotations: {
              bold: false,
              italic: false,
              strikethrough: false,
              underline: false,
              code: false,
              color: "rainbow",
            },
            plain_text: "bad",
            href: null,
          },
        ],
        "rich_text",
      ),
    ).toThrow("rich_text[0].annotations.color must be a supported Notion color");
  });

  it("rejects unsafe linked text URLs", () => {
    expect(() =>
      parseNotesRichTextArray(
        [
          {
            type: "text",
            text: {
              content: "bad",
              link: { url: "javascript:alert(1)" },
            },
            annotations: {
              bold: false,
              italic: false,
              strikethrough: false,
              underline: false,
              code: false,
              color: "default",
            },
            plain_text: "bad",
            href: null,
          },
        ],
        "rich_text",
      ),
    ).toThrow("rich_text[0].text.link.url must be a valid HTTP, HTTPS, or email URL");
  });

  it("parses date reminder rich text", () => {
    const richText = parseNotesRichTextArray(
      [
        {
          type: "mention",
          mention: {
            type: "date",
            date: {
              start: "2026-06-30",
              end: null,
              time_zone: null,
              ganbaru_reminder: { enabled: true },
            },
          },
          annotations: {
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            code: false,
            color: "default",
          },
          plain_text: "Remind Today",
          href: null,
        },
      ],
      "rich_text",
    );

    expect(richText[0]).toMatchObject({
      type: "mention",
      mention: {
        type: "date",
        date: {
          start: "2026-06-30",
          ganbaru_reminder: { enabled: true },
        },
      },
    });
  });

  it("rejects invalid date mention starts", () => {
    expect(() =>
      parseNotesRichTextArray(
        [
          {
            type: "mention",
            mention: {
              type: "date",
              date: { start: "2026-99-30" },
            },
            annotations: {
              bold: false,
              italic: false,
              strikethrough: false,
              underline: false,
              code: false,
              color: "default",
            },
            plain_text: "Bad date",
            href: null,
          },
        ],
        "rich_text",
      ),
    ).toThrow("rich_text[0].mention.date.start must be an ISO date or date-time");
  });

  it("parses expanded mention rich text targets", () => {
    const richText = parseNotesRichTextArray(
        [
          {
            type: "mention",
            mention: {
              type: "user",
              user: {
                object: "user",
                id: "11111111-1111-4111-8111-111111111111",
              },
            },
            annotations: {
              bold: false,
              italic: false,
              strikethrough: false,
              underline: false,
              code: false,
              color: "default",
            },
            plain_text: "Victor",
            href: null,
          },
          {
            type: "mention",
            mention: {
              type: "database",
              database: { id: "22222222-2222-4222-8222-222222222222" },
            },
            annotations: {
              bold: false,
              italic: false,
              strikethrough: false,
              underline: false,
              code: false,
              color: "default",
            },
            plain_text: "Database",
            href: null,
          },
          {
            type: "mention",
            mention: {
              type: "ganbaru_object",
              ganbaru_object: {
                type: "music_item",
                id: "local:/home/victor/Music/focus.mp3",
              },
            },
            annotations: {
              bold: false,
              italic: false,
              strikethrough: false,
              underline: false,
              code: false,
              color: "default",
            },
            plain_text: "Focus",
            href: null,
          },
        ],
        "rich_text",
    );

    expect(richText.map((item) => item.type === "mention" ? item.mention.type : item.type))
      .toEqual(["user", "database", "ganbaru_object"]);
  });

  it("rejects unsupported mention rich text targets", () => {
    expect(() =>
      parseNotesRichTextArray(
        [
          {
            type: "mention",
            mention: {
              type: "link_preview",
              link_preview: { url: "https://example.com" },
            },
            annotations: {
              bold: false,
              italic: false,
              strikethrough: false,
              underline: false,
              code: false,
              color: "default",
            },
            plain_text: "Example",
            href: null,
          },
        ],
        "rich_text",
      ),
    ).toThrow("rich_text[0].mention.type is unsupported");
  });

  it("parses search result DTOs", () => {
    const result = parseNotesSearchResult({
      object: "search_result",
      id: "block:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      type: "block",
      page: { ...basePage, icon: null },
      block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      block_type: "paragraph",
      comment_id: null,
      discussion_id: null,
      comment_status: null,
      comment_author: null,
      comment_anchor: null,
      snippet: "Target block",
      last_edited_time: "2026-06-30T12:00:00.000Z",
    });

    expect(result.type).toBe("block");
    expect(result.block_type).toBe("paragraph");
  });

  it("parses page alias and unresolved link DTOs", () => {
    const alias = parseNotesPageAlias({
      object: "page_alias",
      id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      page_id: basePage.id,
      alias: "Legacy Target",
      normalized_alias: "legacy target",
      created_time: "2026-06-30T12:00:00.000Z",
      last_edited_time: "2026-06-30T12:00:00.000Z",
    });
    const unresolvedLink = parseNotesUnresolvedLink({
      object: "unresolved_link",
      id: "unresolved:block:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa:abc",
      source_type: "block",
      source_page_id: basePage.id,
      source_block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      source_comment_id: null,
      raw_url: "http://localhost:1420/?view=notes#notes?alias=Legacy%20Target",
      raw_target: "Legacy Target",
      normalized_target: "legacy target",
      link_text: "Legacy Target",
      snippet: "Legacy Target",
      created_time: "2026-06-30T12:00:00.000Z",
      last_edited_time: "2026-06-30T12:00:00.000Z",
    });

    expect(alias.alias).toBe("Legacy Target");
    expect(unresolvedLink.source_type).toBe("block");
    expect(unresolvedLink.raw_target).toBe("Legacy Target");
  });

  it("rejects unsupported unresolved link source types", () => {
    expect(() =>
      parseNotesUnresolvedLink({
        object: "unresolved_link",
        id: "bad",
        source_type: "page",
        source_page_id: basePage.id,
        source_block_id: null,
        source_comment_id: null,
        raw_url: "http://localhost:1420/?view=notes#notes?alias=Legacy%20Target",
        raw_target: "Legacy Target",
        normalized_target: "legacy target",
        link_text: "",
        snippet: "",
        created_time: "2026-06-30T12:00:00.000Z",
        last_edited_time: "2026-06-30T12:00:00.000Z",
      }),
    ).toThrow("unresolved_link.source_type must be block or comment");
  });

  it("parses comment search metadata", () => {
    const result = parseNotesSearchResult({
      object: "search_result",
      id: "comment:10101010-1010-4010-8010-101010101010",
      type: "comment",
      page: { ...basePage, icon: null },
      block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      block_type: null,
      comment_id: "10101010-1010-4010-8010-101010101010",
      discussion_id: "90909090-9090-4090-8090-909090909090",
      comment_status: "resolved",
      comment_author: { type: "user", resolved_name: "Reviewer" },
      comment_anchor: {
        object: "comment_anchor",
        type: "text_range",
        block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
        start: 6,
        end: 10,
        text: "beta",
        prefix: "Alpha ",
        suffix: " gamma",
        created_time: "2026-06-30T12:00:00.000Z",
        last_edited_time: "2026-06-30T12:00:00.000Z",
      },
      snippet: "Reviewer beta comment",
      last_edited_time: "2026-06-30T12:00:00.000Z",
    });

    expect(result.comment_status).toBe("resolved");
    expect(result.comment_author?.resolved_name).toBe("Reviewer");
    expect(result.comment_anchor?.text).toBe("beta");
  });

  it("parses property-backed page search result DTOs", () => {
    const result = parseNotesSearchResult({
      object: "search_result",
      id: "page:11111111-1111-4111-8111-111111111111",
      type: "page",
      page: { ...basePage, icon: null },
      block_id: null,
      block_type: null,
      comment_id: null,
      discussion_id: null,
      comment_status: null,
      comment_author: null,
      comment_anchor: null,
      snippet: "Formula state Ready Searchable",
      last_edited_time: "2026-06-30T12:00:00.000Z",
    });

    expect(result.type).toBe("page");
    expect(result.block_id).toBeNull();
    expect(result.snippet).toBe("Formula state Ready Searchable");
  });

  it("rejects unsupported search result types", () => {
    expect(() =>
      parseNotesSearchResult({
        object: "search_result",
        id: "bad",
        type: "database",
        page: { ...basePage, icon: null },
        block_id: null,
        block_type: null,
        comment_id: null,
        discussion_id: null,
        comment_status: null,
        comment_author: null,
        comment_anchor: null,
        snippet: "",
        last_edited_time: "2026-06-30T12:00:00.000Z",
      }),
    ).toThrow("search_result.type must be page, block, or comment");
  });

  it("parses HTML import result DTOs", () => {
    const result = parseNotesHtmlImportResult({
      page: {
        page: { ...basePage, source_provider: "html", source_object_id: "import.html" },
        blocks: {
          object: "list",
          type: "block",
          block: {},
          results: [
            {
              ...baseBlock,
              type: "paragraph",
              paragraph: {
                rich_text: [baseRichText],
                color: "default",
              },
            },
          ],
          next_cursor: null,
          has_more: false,
        },
      },
      diagnostics: [
        {
          code: "html_markup_sanitized",
          severity: "info",
          line: null,
          message: "Unsafe or unsupported HTML markup was removed before import.",
        },
      ],
      imported_block_count: 1,
    });

    expect(result.page.page.source_provider).toBe("html");
    expect(result.diagnostics[0]?.code).toBe("html_markup_sanitized");
    expect(result.imported_block_count).toBe(1);
  });

  it("parses Notion API import result DTOs", () => {
    const result = parseNotesNotionApiImportResult({
      object: "notes_notion_api_import",
      imported_pages: [
        {
          page: { ...basePage, source_provider: "notion", source_object_id: "source-page" },
          blocks: {
            object: "list",
            type: "block",
            block: {},
            results: [
              {
                ...baseBlock,
                type: "paragraph",
                paragraph: {
                  rich_text: [baseRichText],
                  color: "default",
                },
                source_provider: "notion",
                source_object_id: "source-block",
              },
            ],
            next_cursor: null,
            has_more: false,
          },
        },
      ],
      imported_data_sources: [
        {
          object_type: "data_source",
          source_object_id: "source-data-source",
          local_id: "81818181-8181-4181-8181-818181818181",
          title: "Tasks",
        },
      ],
      imported_users: [
        {
          source_user_id: "notion-user",
          name: "Avo Cado",
          user_type: "person",
        },
      ],
      diagnostics: [
        {
          code: "temporary_notion_file_url",
          severity: "warning",
          source_object_id: "source-block",
          message: "File URL may expire.",
        },
      ],
      request_count: 6,
      retry_count: 1,
      rate_limit_count: 1,
      imported_page_count: 1,
      imported_block_count: 1,
      imported_data_source_count: 1,
      imported_comment_count: 2,
      imported_user_count: 1,
      imported_file_count: 1,
      unsupported_block_count: 0,
    });

    expect(result.object).toBe("notes_notion_api_import");
    expect(result.imported_pages[0]?.page.source_provider).toBe("notion");
    expect(result.imported_data_sources[0]?.local_id).toBe("81818181-8181-4181-8181-818181818181");
    expect(result.imported_users[0]?.name).toBe("Avo Cado");
    expect(result.diagnostics[0]?.code).toBe("temporary_notion_file_url");
    expect(result.rate_limit_count).toBe(1);
  });

  it("parses markdown export result DTOs", () => {
    const result = parseNotesMarkdownExportResult({
      object: "notes_markdown_export",
      page_id: "11111111-1111-4111-8111-111111111111",
      markdown: "# Export\n",
      diagnostics: [
        {
          code: "markdown_export_unsupported_block",
          severity: "warning",
          block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
          comment_id: null,
          message: "Unsupported block content was preserved",
        },
      ],
      exported_block_count: 2,
      exported_comment_count: 1,
    });

    expect(result.object).toBe("notes_markdown_export");
    expect(result.diagnostics[0]?.block_id).toBe("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    expect(result.exported_comment_count).toBe(1);
  });

  it("parses HTML archive export result DTOs", () => {
    const result = parseNotesHtmlExportResult({
      object: "notes_html_archive_export",
      root_page_id: "11111111-1111-4111-8111-111111111111",
      files: [
        {
          path: "index.html",
          content_type: "text/html; charset=utf-8",
          contents: "<!doctype html>",
          byte_size: 15,
        },
      ],
      assets: [
        {
          id: "notes/files/local.txt",
          archive_path: "assets/notes/files/local.txt",
          source_path: "notes/files/local.txt",
          content_type: "text/plain",
          byte_size: 12,
          sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          storage_state: "available",
          exported: true,
        },
      ],
      diagnostics: [
        {
          code: "html_export_local_asset_not_included",
          severity: "warning",
          page_id: "11111111-1111-4111-8111-111111111111",
          block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
          asset_id: null,
          comment_id: null,
          message: "Local managed asset was not included in this archive",
        },
      ],
      manifest_json: "{\"object\":\"notes_html_archive_manifest\"}",
      exported_page_count: 1,
      exported_block_count: 2,
      exported_asset_count: 1,
      exported_comment_count: 0,
      exported_database_view_count: 0,
    });

    expect(result.object).toBe("notes_html_archive_export");
    expect(result.files[0]?.path).toBe("index.html");
    expect(result.assets[0]?.exported).toBe(true);
    expect(result.diagnostics[0]?.block_id).toBe("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    expect(result.exported_asset_count).toBe(1);
  });

  it("parses HTML archive save result DTOs", () => {
    const result = parseNotesHtmlArchiveSaveResult({
      object: "notes_html_archive_save",
      saved: false,
      export: null,
    });

    expect(result.saved).toBe(false);
    expect(result.export).toBeNull();
  });

  it("parses data source CSV import result DTOs", () => {
    const result = parseNotesDataSourceCsvImportResult({
      object: "notes_data_source_csv_import",
      data_source_id: "81818181-8181-4181-8181-818181818181",
      dry_run: true,
      total_row_count: 2,
      valid_row_count: 1,
      skipped_row_count: 1,
      imported_row_count: 0,
      imported_page_ids: [],
      columns: [
        {
          source_index: 1,
          source_name: "Name",
          property_id: "title",
          property_name: "Name",
          property_type: "title",
          mapped: true,
          read_only: false,
          warning: null,
        },
        {
          source_index: 2,
          source_name: "Ticket",
          property_id: "ticket",
          property_name: "Ticket",
          property_type: "unique_id",
          mapped: true,
          read_only: true,
          warning: "CSV column maps to a read-only property and will be skipped.",
        },
      ],
      rows: [
        {
          row_number: 2,
          title: "Alpha",
          valid: true,
          mapped_cell_count: 1,
          error_count: 0,
        },
        {
          row_number: 3,
          title: "Broken",
          valid: false,
          mapped_cell_count: 1,
          error_count: 1,
        },
      ],
      diagnostics: [
        {
          code: "invalid_cell",
          severity: "error",
          row_number: 3,
          column_index: 3,
          column_name: "Estimate",
          property_id: "estimate",
          message: "number value must be a valid number",
        },
      ],
    });

    expect(result.object).toBe("notes_data_source_csv_import");
    expect(result.valid_row_count).toBe(1);
    expect(result.columns[1]?.read_only).toBe(true);
    expect(result.rows[1]?.valid).toBe(false);
    expect(result.diagnostics[0]?.property_id).toBe("estimate");
  });

  it("parses data source CSV export save result DTOs", () => {
    const result = parseNotesDataSourceCsvExportSaveResult({
      saved: true,
      export: {
        object: "notes_data_source_csv_export",
        data_source_id: "81818181-8181-4181-8181-818181818181",
        database_id: "82828282-8282-4282-8282-828282828282",
        view_id: "83838383-8383-4383-8383-838383838383",
        scope: "view",
        file_name: "tasks-table.csv",
        csv: "Name\nAlpha\n",
        exported_row_count: 1,
        exported_property_count: 1,
        diagnostics: [
          {
            code: "csv_export_plain_text_property",
            severity: "warning",
            property_id: "files",
            property_name: "Files",
            message: "Files is exported as plain text.",
          },
        ],
      },
    });

    expect(result.saved).toBe(true);
    expect(result.export?.scope).toBe("view");
    expect(result.export?.csv).toBe("Name\nAlpha\n");
    expect(result.export?.diagnostics[0]?.property_id).toBe("files");
  });
});
