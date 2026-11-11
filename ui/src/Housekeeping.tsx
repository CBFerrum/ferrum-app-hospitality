// Housekeeping: the rooms to clean, oldest first: departures to make ready for the next guest and
// stayovers to service. Cleaned, a departure waits for a supervisor (not whoever cleaned it) to
// inspect it, which is what a hotel that inspects its rooms checks guests into.

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, docPath, refusal, t, isolate } from "@ferrum/client";
import { Button, HomeSkeleton, LoadFailed, PageHead, Queue, QueueGrid, QueueItem, StatePanel, StatusBadge, toast, type Tone } from "@ferrum/kit";

export interface Task {
  id: string;
  room_number: string;
  task_date: string;
  kind: "Departure" | "Stayover";
  status: "To Do" | "Cleaned" | "Inspected";
  assigned_to?: string | null;
  cleaned_by?: string | null;
}

/** HousekeepingTask.status as stored, for `statusTone`: still to do, or cleaned and waiting. */
export const TASK_TONES: Readonly<Record<string, Tone>> = { "To Do": "amber", Cleaned: "green", Inspected: "green" };

/** What a task's next step is: cleaning it, then (a departure only) inspecting it. */
export function nextStep(task: Pick<Task, "kind" | "status">): { action: string; label: string } | null {
  if (task.status === "To Do") return { action: "cleaned", label: t("Cleaned") };
  if (task.status === "Cleaned" && task.kind === "Departure") return { action: "inspect", label: t("Inspected") };
  return null;
}

export function Housekeeping() {
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
  if (q.isPending)
    return (
      <>
        <PageHead title={t("Housekeeping")} back={{ to: "/", label: t("Front desk") }} />
        <HomeSkeleton queues={2} />
      </>
    );
  if (q.isError)
    return (
      <>
        <PageHead title={t("Housekeeping")} back={{ to: "/", label: t("Front desk") }} />
        <LoadFailed error={q.error} onRetry={() => void q.refetch()} />
      </>
    );
  // A task is in the list it is in on the server: a step taken moves it only once the server has answered and the list is read again.
  const todo = q.data.filter((x) => x.status === "To Do");
  const cleaned = q.data.filter((x) => x.status === "Cleaned");
  // The button names the step as an action ("Mark cleaned"), so it is not mistaken for the state beside it.
  const stepWord = (action: string) => (action === "cleaned" ? t("Mark cleaned") : t("Mark inspected"));
  const row = (task: Task) => {
    const next = nextStep(task);
    return (
      <QueueItem
        key={task.id}
        title={t("Room {room}", { room: task.room_number })}
        meta={`${isolate(task.kind)} · ${isolate(task.task_date)}${task.assigned_to ? ` · ${task.assigned_to}` : ""}`}
        trailing={<StatusBadge value={task.status} tones={TASK_TONES} />}
        action={
          next ? (
            <Button variant="primary" loading={step.isPending && step.variables?.id === task.id} onClick={() => step.mutate({ id: task.id, action: next.action })}>
              {stepWord(next.action)}
            </Button>
          ) : undefined
        }
      />
    );
  };
  return (
    <>
      <PageHead title={t("Housekeeping")} description={t("{n} to do", { n: todo.length })} back={{ to: "/", label: t("Front desk") }} />
      {q.data.length === 0 ? (
        <StatePanel kind="empty" title={t("Every room is done")}>
          {t("Check-outs and the night audit put rooms here to clean.")}
        </StatePanel>
      ) : (
        <QueueGrid>
          <Queue title={t("To clean")} count={todo.length}>
            {todo.map(row)}
          </Queue>
          <Queue title={t("Cleaned")} count={cleaned.length}>
            {cleaned.map(row)}
          </Queue>
        </QueueGrid>
      )}
    </>
  );
}
