// The public booking page's chrome: no bar and no sign-in (the platform serves it as a public page),
// just the hotel group's name over the page.

import type { ReactNode } from "react";
import { ApiError, bootstrap, t, isolate } from "@ferrum/client";
import { EmptyState, ErrorState } from "@ferrum/kit";

export function Frame({ children }: { children: ReactNode }) {
  const name = bootstrap()?.workspace.name;
  return (
    <div className="hb-page">
      <a className="skip-link" href="#main">
        {t("Skip to content")}
      </a>
      <header className="hb-top">
        <span className="hb-brand">{isolate(name ?? t("Book a room"))}</span>
      </header>
      <main id="main" className="hb-main" tabIndex={-1}>
        {children}
      </main>
    </div>
  );
}

/** A page that does not work: no property books online, or the link is wrong or has run out. */
export function Closed({ error, what, onRetry }: { error: unknown; what: "page" | "link"; onRetry?: () => void }) {
  if (error instanceof ApiError && error.status === 404) {
    return what === "page" ? (
      <EmptyState title={t("Rooms are not booked online here")} icon="calendar">
        <p className="muted">{t("The address may not be quite right. Please get in touch with the hotel directly.")}</p>
      </EmptyState>
    ) : (
      <EmptyState title={t("This link does not work")} icon="calendar">
        <p className="muted">{t("A reservation's link stops working a month after the stay, and opens only the reservation it was made for.")}</p>
      </EmptyState>
    );
  }
  return <ErrorState error={error} onRetry={onRetry} title={t("We could not load this page")} />;
}
