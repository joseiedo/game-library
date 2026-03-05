import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import GameCard from "./GameCard.vue";
import type { Game } from "../types/game";

const steamGame: Game = {
  key: "steam-440",
  title: "Team Fortress 2",
  platform: "steam",
  coverImage: "https://cdn.cloudflare.steamstatic.com/steam/apps/440/library_600x900.jpg",
  appId: 440,
  isShortcut: false,
  tags: [],
};

const customGame: Game = {
  key: "custom-abc",
  title: "My Custom Game",
  platform: "custom",
  coverImage: null,
  executable: "/games/mygame.exe",
  tags: ["action"],
};

const epicGame: Game = {
  key: "epic-Fortnite",
  title: "Fortnite",
  platform: "epic",
  coverImage: null,
  epicLaunchUri: "com.epicgames.launcher://apps/...",
  tags: [],
};

describe("GameCard", () => {
  describe("rendering", () => {
    it("displays the game title", () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: false } });
      expect(wrapper.text()).toContain("Team Fortress 2");
    });

    it("displays Steam platform badge", () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: false } });
      expect(wrapper.text()).toContain("Steam");
    });

    it("displays Epic platform badge", () => {
      const wrapper = mount(GameCard, { props: { game: epicGame, focused: false } });
      expect(wrapper.text()).toContain("Epic");
    });

    it("displays Custom platform badge", () => {
      const wrapper = mount(GameCard, { props: { game: customGame, focused: false } });
      expect(wrapper.text()).toContain("Custom");
    });

    it("renders cover image when present", () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: false } });
      const img = wrapper.find("img");
      expect(img.exists()).toBe(true);
      expect(img.attributes("src")).toBe(steamGame.coverImage);
      expect(img.attributes("alt")).toBe("Team Fortress 2");
    });

    it("renders placeholder icon when no cover image", () => {
      const wrapper = mount(GameCard, { props: { game: customGame, focused: false } });
      expect(wrapper.find("img").exists()).toBe(false);
      expect(wrapper.find("svg").exists()).toBe(true);
    });

    it("applies focused ring class when focused", () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: true } });
      const root = wrapper.find("[role='button']");
      expect(root.classes()).toContain("ring-2");
    });

    it("does not apply ring class when not focused", () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: false } });
      const root = wrapper.find("[role='button']");
      expect(root.classes()).not.toContain("ring-2");
    });
  });

  describe("context menu (custom games only)", () => {
    it("shows three-dot menu button for custom games", () => {
      const wrapper = mount(GameCard, { props: { game: customGame, focused: false } });
      expect(wrapper.find("[aria-label='Game options']").exists()).toBe(true);
    });

    it("does not show three-dot menu for steam games", () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: false } });
      expect(wrapper.find("[aria-label='Game options']").exists()).toBe(false);
    });

    it("does not show three-dot menu for epic games", () => {
      const wrapper = mount(GameCard, { props: { game: epicGame, focused: false } });
      expect(wrapper.find("[aria-label='Game options']").exists()).toBe(false);
    });
  });

  describe("events", () => {
    it("emits launch on click", async () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: false } });
      await wrapper.find("[role='button']").trigger("click");
      expect(wrapper.emitted("launch")).toHaveLength(1);
    });

    it("emits focus when the card receives focus", async () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: false } });
      await wrapper.find("[role='button']").trigger("focus");
      expect(wrapper.emitted("focus")).toHaveLength(1);
    });

    it("emits launch on Enter keydown", async () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: false } });
      await wrapper.find("[role='button']").trigger("keydown", { key: "Enter" });
      expect(wrapper.emitted("launch")).toHaveLength(1);
    });

    it("emits launch on Space keydown", async () => {
      const wrapper = mount(GameCard, { props: { game: steamGame, focused: false } });
      await wrapper.find("[role='button']").trigger("keydown", { key: " " });
      expect(wrapper.emitted("launch")).toHaveLength(1);
    });

    it("emits edit when edit menu item clicked", async () => {
      const wrapper = mount(GameCard, { props: { game: customGame, focused: false } });
      await wrapper.find("[aria-label='Game options']").trigger("click");
      await wrapper.find("button[class*='text-zinc-700']").trigger("click");
      expect(wrapper.emitted("edit")).toHaveLength(1);
    });

    it("emits delete when delete menu item clicked", async () => {
      const wrapper = mount(GameCard, { props: { game: customGame, focused: false } });
      await wrapper.find("[aria-label='Game options']").trigger("click");
      await wrapper.find("button[class*='text-red-600']").trigger("click");
      expect(wrapper.emitted("delete")).toHaveLength(1);
    });
  });
});
