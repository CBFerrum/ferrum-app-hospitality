// Housekeeping: the rooms to clean, oldest first: departures to make ready for the next guest and
// stayovers to service. Cleaned, a departure waits for a supervisor (not whoever cleaned it) to
// inspect it, which is what a hotel that inspects its rooms checks guests into.

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, docPath, refusal, t, isolate } from "@ferrum/client";
import { Badge, Button, EmptyState, ErrorState, ListSkeleton, toast, useShellTitle } from "@ferrum/kit";

export interface Task {
  id: string;
  room_number: string;
  task_date: string;
  kind: "Departure" | "Stayover";
  status: "To Do" | "Cleaned" | "Inspected";
  assigned_to?: string | null;
  cleaned_by?: string | null;
}

/** What a task's next step is: cleaning it, then (a departure only) inspecting it. */
export function nextStep(task: Pick<Task, "kind" | "status">): { action: string; label: string } | null {
  if (task.status === "To Do") return { action: "cleaned", label: t("Cleaned") };
  if (task.status === "Cleaned" && task.kind === "Departure") return { action: "inspect", label: t("Inspected") };
  return null;
}

export function Housekeeping() {
  useShellTitle(t("Housekeeping"));
  const qc = useQueryClient();
  const q = useQuery({
    queryKey: ["hotel", "housekeeping"],
    queryFn: () =>
      api
        .get<{ rows: Task[] }>(docPath("HousekeepingTask"), {
          filters: JSON.stringify([["status", "in", ["To Do", "Cleaned"]]]),
          sort: "task_date asc",
          limit: 500,
        })
        .then((r) => r.rows),
    refetchInterval: 30_000,
  });
  const step = useMutation({
    mutationFn: ({ id, action }: { id: string; action: string }) => api.post<{ message?: string }>(docPath("HousekeepingTask", id, "actions", action), {}),
    onSuccess: (r) => {
      if (r.message) toast(r.message);
      void qc.invalidateQueries({ queryKey: ["hotel"] });
    },
    onError: (e) => toast(refusal(e), "error"),
  });
  if (q.isPending) return <ListSkeleton className="hotel-page" />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  return (
    <div className="hotel-page">
      <h2>{t("Housekeeping")}</h2>
      {q.data.length === 0 ? (
        <EmptyState title={t("Every room is done")} icon="list">
          <p className="muted">{t("Check-outs and the night audit put rooms here to clean.")}</p>
        </EmptyState>
      ) : (
        <div className="hotel-columns">
          {q.data.map((task) => {
            const next = nextStep(task);
            return (
              <section key={task.id} className="card card-pad stack" aria-label={t("Room {room}", { room: task.room_number })}>
                <div className="row" style={{ justifyContent: "space-between" }}>
                  <strong>{t("Room {room}", { room: task.room_number })}</strong>
                  <Badge tone={task.status === "Cleaned" ? "green" : "amber"}>{isolate(task.status)}</Badge>
                </div>
                <div className="faint">
                  {isolate(task.kind)} · {isolate(task.task_date)}
                  {task.assigned_to ? ` · ${task.assigned_to}` : ""}
                </div>
                {next && (
                  <Button size="sm" loading={step.isPending && step.variables?.id === task.id} onClick={() => step.mutate({ id: task.id, action: next.action })}>
                    {isolate(next.label)}
                  </Button>
                )}
              </section>
            );
          })}
        </div>
      )}
    </div>
  );
}
