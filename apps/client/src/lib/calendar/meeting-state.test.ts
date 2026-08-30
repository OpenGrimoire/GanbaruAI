import { describe, expect, it } from "vitest";
import {
  hasMeetingState,
  hasNonDefaultGuestPermissions,
} from "./meeting-state";

describe("calendar meeting state", () => {
  it("treats explicit meeting enablement and meeting metadata as meeting state", () => {
    expect(hasMeetingState({ meetingEnabled: true })).toBe(true);
    expect(hasMeetingState({
      attendees: [{
        id: "attendee-1",
        email: "person@example.com",
        role: "req-participant",
        status: "needs-action",
        rsvp: false,
      }],
    })).toBe(true);
    expect(hasMeetingState({ organizer: { email: "owner@example.com" } })).toBe(true);
    expect(hasMeetingState({ location: "Room A" })).toBe(true);
    expect(hasMeetingState({ url: "https://example.com/call" })).toBe(true);
    expect(hasMeetingState({ geo: { lat: 1, lng: 2 } })).toBe(true);
    expect(hasMeetingState({ localParticipationStatus: "accepted" })).toBe(true);
  });

  it("recognizes only guest permissions that differ from defaults", () => {
    expect(hasNonDefaultGuestPermissions(undefined)).toBe(false);
    expect(hasNonDefaultGuestPermissions({
      canModify: false,
      canInviteOthers: true,
      canSeeOtherGuests: true,
    })).toBe(false);
    expect(hasMeetingState({
      guestPermissions: {
        canModify: true,
        canInviteOthers: true,
        canSeeOtherGuests: true,
      },
    })).toBe(true);
    expect(hasMeetingState({
      guestPermissions: {
        canModify: false,
        canInviteOthers: false,
        canSeeOtherGuests: true,
      },
    })).toBe(true);
    expect(hasMeetingState({
      guestPermissions: {
        canModify: false,
        canInviteOthers: true,
        canSeeOtherGuests: false,
      },
    })).toBe(true);
  });

  it("does not create meeting state from empty or default values", () => {
    expect(hasMeetingState({})).toBe(false);
    expect(hasMeetingState({
      meetingEnabled: false,
      attendees: [],
      location: "",
      url: "",
      guestPermissions: {
        canModify: false,
        canInviteOthers: true,
        canSeeOtherGuests: true,
      },
    })).toBe(false);
  });
});
