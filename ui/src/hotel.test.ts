import { describe, expect, it } from "vitest";
import { statusTone } from "@ferrum/kit";
import { ROOM_TONES, roomNumber, shortDay } from "./FrontDesk";
import { dayAfter, manageLink } from "./guest/site";
import { nextStep, TASK_TONES } from "./Housekeeping";
describe("housekeeping and the booking page", () => {
  it("cleans a room, then inspects a departure only", () => {
    expect(nextStep({ kind: "Departure", status: "To Do" })?.action).toBe("cleaned");
    expect(nextStep({ kind: "Departure", status: "Cleaned" })?.action).toBe("inspect");
    expect(nextStep({ kind: "Stayover", status: "Cleaned" })).toBeNull();
    expect(nextStep({ kind: "Departure", status: "Inspected" })).toBeNull();
  });

  it("starts the search tomorrow, in the guest's own calendar", () => {
    expect(dayAfter(new Date(2026, 11, 31), 1)).toBe("2027-01-01");
    expect(dayAfter(new Date(2026, 1, 28), 2)).toBe("2026-03-02");
  });

  it("keeps a guest's link under the booking page", () => {
    expect(manageLink("harbor hotel", "a/b")).toBe("/book/harbor%20hotel/manage/a%2Fb");
  });
});

describe("the desk's rooms and badges", () => {
  it("tones every stored room state differently, and a state it lacks as neutral", () => {
    const tones = Object.values(ROOM_TONES);
    expect(Object.keys(ROOM_TONES).sort()).toEqual(["Clean", "Dirty", "Inspected", "Out of Order"]);
    expect(new Set(tones).size).toBe(tones.length);
    expect(statusTone("Out of Order", ROOM_TONES)).toBe("red");
    expect(statusTone("Something new", ROOM_TONES)).toBe("neutral");
  });

  it("tones a housekeeping task by where it stands", () => {
    expect(statusTone("To Do", TASK_TONES)).toBe("amber");
    expect(statusTone("Cleaned", TASK_TONES)).toBe("green");
  });

  it("names a stay's room by number, and nothing before one is given", () => {
    const rooms = [{ id: "r1", room_number: "101" }];
    expect(roomNumber({ room: "r1" }, rooms)).toBe("101");
    expect(roomNumber({ room: null }, rooms)).toBe("");
    expect(roomNumber({}, rooms)).toBe("");
  });

  it("shows a stay's day in a row without its year", () => {
    expect(shortDay("2026-11-14")).toMatch(/14/);
    expect(shortDay("2026-11-14")).not.toMatch(/2026/);
  });
});
