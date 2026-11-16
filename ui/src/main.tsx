// Hospitality's own interface at /hotel/: the front desk (who arrives, who is in, who leaves, and
// the rooms) and housekeeping, with every record in the records frame's lists and forms. And the
// public booking page guests use, at /hotel/book/{workspace}: a page anyone with the address opens
// (`shell.public`), so it is its own route tree and never asks anyone to sign in.

import { useEffect } from "react";
import { Link, Outlet, useRouteError } from "react-router";
import { shell, t } from "@ferrum/client";
import { EmptyState, ErrorState } from "@ferrum/kit";
import { mountApp, RequireAuth, setFrameMode } from "@ferrum/records/frame";
import "./styles.css";

const page = (name: "FormView" | "ListView" | "ReportView") => () => import("./records").then((m) => ({ Component: m[name] }));

function RouteError() {
  return <ErrorState error={useRouteError()} onRetry={() => window.location.reload()} />;
}

function NotFound() {
  return (
    <EmptyState title={t("Page not found")}>
      <Link to="/">{t("Go to the front desk")}</Link>
    </EmptyState>
  );
}

/** Every page tells the shell it started. */
function Started() {
  useEffect(() => shell().ready(), []);
  return null;
}

/** A guest's page: started, and drawn below without the frame. */
function Public() {
  useEffect(() => shell().ready(), []);
  return <Outlet />;
}

setFrameMode("app", ["hospitality"]);

mountApp({
  route: "hotel",
  catalogs: { ar: () => import("./locales/ar.json"), hi: () => import("./locales/hi.json") },
  routes: [
    // For guests: no sign-in, no frame.
    { path: "/book/:tenant", element: <Public />, errorElement: <RouteError />, children: [{ index: true, lazy: () => import("./guest/Book").then((m) => ({ Component: m.BookPage })) }] },
    {
      path: "/book/:tenant/manage/:token",
      element: <Public />,
      errorElement: <RouteError />,
      children: [{ index: true, lazy: () => import("./guest/Manage").then((m) => ({ Component: m.ManageRoute })) }],
    },
    { path: "/book/*", element: <EmptyState title={t("Page not found")} icon="calendar" /> },
    {
      path: "/",
      element: (
        <>
          <Started />
          <RequireAuth />
        </>
      ),
      errorElement: <RouteError />,
      children: [
        { index: true, lazy: () => import("./FrontDesk").then((m) => ({ Component: m.FrontDesk })) },
        { path: "housekeeping", lazy: () => import("./Housekeeping").then((m) => ({ Component: m.Housekeeping })) },
        { path: "e/:entity", lazy: page("ListView") },
        { path: "e/:entity/:id", lazy: page("FormView") },
        { path: "r/:name", lazy: page("ReportView") },
        { path: "*", element: <NotFound /> },
      ],
    },
  ],
});
