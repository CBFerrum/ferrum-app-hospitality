// The booking page's endpoints (`/api/site/{workspace}/hospitality/…`, the app's site handler): what a
// guest can ask without signing in. The server holds every rule; these only carry the question.

import { api } from "@ferrum/client";

export interface Property {
  code: string;
  name: string;
  address: string | null;
  check_in_time: string | null;
  check_out_time: string | null;
}

export interface Offer {
  room_type: string;
  name: string;
  sleeps: number;
  free: number;
  total: string;
  deposit: string;
  amenities: string[];
}

export interface Offers {
  property: string;
  arrival: string;
  departure: string;
  nights: number;
  /** Days before arrival a cancellation still gets its deposit back; null: any time before. */
  refund_days: number | null;
  offers: Offer[];
}

export interface Reservation {
  number: string;
  property: string;
  room_type: string | null;
  guest_name: string;
  arrival: string;
  departure: string;
  nights: number;
  adults: number;
  children: number;
  check_in_time: string | null;
  check_out_time: string | null;
  total: string;
  deposit_due: string | null;
  deposit_paid: string | null;
  deposit_outcome: "Refund Due" | "Kept" | null;
  refund_days: number | null;
  status: string;
  cancellable: boolean;
}

export interface Search {
  property: string;
  arrival: string;
  departure: string;
  adults: number;
  children: number;
}

const base = (tenant: string) => `/api/site/${encodeURIComponent(tenant)}/hospitality`;

export const properties = (tenant: string, signal?: AbortSignal) => api.get<{ properties: Property[] }>(`${base(tenant)}/properties`, undefined, signal);

export const availability = (tenant: string, q: Search, signal?: AbortSignal) => api.get<Offers>(`${base(tenant)}/availability`, { ...q }, signal);

/** A reservation and the guest's link to it, or `{status: "received"}` for the field only a program fills in. */
export type Booked = { reservation: Reservation; token: string } | { status: "received" };

export const book = (tenant: string, body: unknown) => api.post<Booked>(`${base(tenant)}/book`, body);

export const mine = (tenant: string, token: string, signal?: AbortSignal) =>
  api.get<{ reservation: Reservation }>(`${base(tenant)}/manage/${encodeURIComponent(token)}`, undefined, signal);

export const cancel = (tenant: string, token: string, reason: string) =>
  api.post<{ reservation: Reservation; message?: string }>(`${base(tenant)}/manage/${encodeURIComponent(token)}/cancel`, { reason: reason.trim() || undefined });

/** The page of a guest's reservation, under the booking page. */
export const manageLink = (tenant: string, token: string) => `/book/${encodeURIComponent(tenant)}/manage/${encodeURIComponent(token)}`;

/** `today` plus `n` days, as YYYY-MM-DD in the guest's own calendar. */
export function dayAfter(today: Date, n: number): string {
  const d = new Date(today.getFullYear(), today.getMonth(), today.getDate() + n);
  const pad = (v: number) => String(v).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

/** What the deposit policy tells a guest before they book. */
export function refundTerms(days: number | null): "any" | "days" {
  return days === null ? "any" : "days";
}
