// Hospitality's own interface at /hotel/: the front desk (who arrives, who is in, who leaves, and
// the rooms) and housekeeping, with every record in the records frame's lists and forms. And the
// public booking page guests use, at /hotel/book/{workspace}: a page anyone with the address opens
// (`shell.public`), so it is its own route tree and never asks anyone to sign in.

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { StrictMode, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { createBrowserRouter, Link, Outlet, RouterProvider, useRouteError } from "react-router";
import { addCatalogs, routeBase, shell, t, wordsReady } from "@ferrum/client";
import { ConfirmHost, EmptyState, ErrorState, Toaster } from "@ferrum/kit";
import { RequireAuth, setFrameMode } from "@ferrum/records/frame";
import "./styles.css";

// Its own words in the page's language (src/locales), fetched as the page starts; it is drawn once they are in.
void addCatalogs({ ar: () => import("./locales/ar.json"), hi: () => import("./locales/hi.json") });

const page = (name: "FormView" | "ListView" | "ReportView" | "AppSettingsPage" | "ScreenPage") => () => import("./records").then((m) => ({ Component: m[name] }));

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false, refetchOnWindowFocus: false, staleTime: 5_000 } } });

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

const router = createBrowserRouter(
  [
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
        { path: "apps/:app/settings", lazy: page("AppSettingsPage") },
        { path: "x/:app/:page", lazy: page("ScreenPage") },
        { path: "*", element: <NotFound /> },
      ],
    },
  ],
  { basename: routeBase("hotel") },
);

// Drawn once the page's words are in (its language's catalogs, fetched as the page starts).
void wordsReady().then(() =>
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
        <Toaster />
        <ConfirmHost />
      </QueryClientProvider>
    </StrictMode>,
  ),
);
