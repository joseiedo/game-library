import { describe, it, expect, vi } from "vitest";
import { fromSteamGame, fromCustomGame, fromEpicGame } from "./game";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `asset://${path}`,
}));

describe("fromSteamGame", () => {
  const raw = {
    app_id: 440,
    name: "Team Fortress 2",
    install_dir: "/games/tf2",
    is_shortcut: false,
  };

  it("builds the correct key", () => {
    expect(fromSteamGame(raw).key).toBe("steam-440");
  });

  it("maps name to title", () => {
    expect(fromSteamGame(raw).title).toBe("Team Fortress 2");
  });

  it("sets platform to steam", () => {
    expect(fromSteamGame(raw).platform).toBe("steam");
  });

  it("generates Steam CDN cover URL", () => {
    expect(fromSteamGame(raw).coverImage).toBe(
      "https://cdn.cloudflare.steamstatic.com/steam/apps/440/library_600x900.jpg",
    );
  });

  it("preserves appId and isShortcut", () => {
    const game = fromSteamGame(raw);
    expect(game.appId).toBe(440);
    expect(game.isShortcut).toBe(false);
  });

  it("starts with empty tags", () => {
    expect(fromSteamGame(raw).tags).toEqual([]);
  });
});

describe("fromCustomGame", () => {
  const raw = {
    id: "abc-123",
    title: "My Game",
    executable: "/games/mygame.exe",
    cover_image: "/images/cover.png",
    tags: ["action", "indie"],
    notes: null,
  };

  it("builds the correct key", () => {
    expect(fromCustomGame(raw).key).toBe("custom-abc-123");
  });

  it("maps title", () => {
    expect(fromCustomGame(raw).title).toBe("My Game");
  });

  it("sets platform to custom", () => {
    expect(fromCustomGame(raw).platform).toBe("custom");
  });

  it("resolves cover image via convertFileSrc", () => {
    expect(fromCustomGame(raw).coverImage).toBe("asset:///images/cover.png");
  });

  it("returns null coverImage when no path given", () => {
    expect(fromCustomGame({ ...raw, cover_image: null }).coverImage).toBeNull();
  });

  it("maps executable", () => {
    expect(fromCustomGame(raw).executable).toBe("/games/mygame.exe");
  });

  it("maps tags", () => {
    expect(fromCustomGame(raw).tags).toEqual(["action", "indie"]);
  });
});

describe("fromEpicGame", () => {
  const raw = {
    app_name: "FortniteNewCaldera",
    display_name: "Fortnite",
    install_location: "/games/fortnite",
    catalog_namespace: "fn",
    catalog_item_id: "item42",
    cover_image: null,
  };

  it("builds the correct key", () => {
    expect(fromEpicGame(raw).key).toBe("epic-FortniteNewCaldera");
  });

  it("maps display_name to title", () => {
    expect(fromEpicGame(raw).title).toBe("Fortnite");
  });

  it("sets platform to epic", () => {
    expect(fromEpicGame(raw).platform).toBe("epic");
  });

  it("generates Epic launcher URI", () => {
    expect(fromEpicGame(raw).epicLaunchUri).toBe(
      "com.epicgames.launcher://apps/fn%3Aitem42%3AFortniteNewCaldera?action=launch&silent=true",
    );
  });

  it("returns null coverImage when no cover path given", () => {
    expect(fromEpicGame(raw).coverImage).toBeNull();
  });

  it("resolves cover image via convertFileSrc when present", () => {
    const game = fromEpicGame({ ...raw, cover_image: "/cache/fortnite.jpg" });
    expect(game.coverImage).toBe("asset:///cache/fortnite.jpg");
  });
});
