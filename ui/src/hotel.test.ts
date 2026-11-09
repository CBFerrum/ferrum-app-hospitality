import { describe, expect, it } from "vitest";
import { dayAfter, manageLink } from "./guest/site";
import { nextStep } from "./Housekeeping";

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
