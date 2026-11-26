import { useEffect, useRef, useState } from "react";
import { AlertTriangle, Loader2, UserMinus } from "lucide-react";
import { previewRelease } from "@/lib/api";
import { usePlayerDetail } from "@/lib/query";
import type { ReleaseChange } from "@/lib/types";
import { useI18n } from "@/lib/useI18n";
import { useAppStore } from "@/store/app";
import { Button, Modal } from "./ui";

export function ReleaseDialog() {
  const { t } = useI18n();
  const open = useAppStore((state) => state.releaseOpen);
  const setOpen = useAppStore((state) => state.setReleaseOpen);
  const saveId = useAppStore((state) => state.saveId);
  const selectedUid = useAppStore((state) => state.selectedUid);
  const pendingEdits = useAppStore((state) => state.pendingEdits);
  const pendingTransfers = useAppStore((state) => state.pendingTransfers);
  const pendingReleases = useAppStore((state) => state.pendingReleases);
  const stageRelease = useAppStore((state) => state.stageRelease);
  const pushToast = useAppStore((state) => state.pushToast);
  const { data: player, isLoading } = usePlayerDetail(open ? selectedUid : null);
  const [plan, setPlan] = useState<ReleaseChange | null>(null);
  const [previewing, setPreviewing] = useState(false);
  const [planError, setPlanError] = useState<string | null>(null);
  const requestId = useRef(0);

  // A new player or a reopened dialog starts a fresh preview.
  useEffect(() => {
    if (!open) return;
    requestId.current += 1;
    setPlan(null);
    setPlanError(null);
    setPreviewing(false);
    if (saveId == null || selectedUid == null) return;
    const request = ++requestId.current;
    setPreviewing(true);
    previewRelease(saveId, selectedUid)
      .then((preview) => {
        if (request === requestId.current) setPlan(preview);
      })
      .catch((error) => {
        if (request === requestId.current) setPlanError(String(error));
      })
      .finally(() => {
        if (request === requestId.current) setPreviewing(false);
      });
  }, [open, saveId, selectedUid]);

  const pendingEdit = player ? pendingEdits[player.uid] : undefined;
  const pendingTransfer = player ? pendingTransfers[player.uid] : undefined;
  const pendingRelease = player ? pendingReleases[player.uid] : undefined;
  const blocked = pendingTransfer != null;

  const stage = () => {
    if (!player || !plan || blocked) return;
    const name = player.name ?? t("common.unnamedPlayer");
    stageRelease({
      uid: player.uid,
      name,
      from_club_name: plan.from_club_name,
      from_team_id: plan.from_team_id,
      players_before: plan.players_before,
      players_after: plan.players_after,
      lists_cleared: plan.lists.length,
      captaincy_cleared: plan.captaincy_cleared,
      changes: plan.changes,
    });
    pushToast({
      title: t("release.staged"),
      description: t("release.stagedDescription", {
        name,
        club: plan.from_club_name,
      }),
      tone: "success",
    });
    setOpen(false);
  };

  return (
    <Modal
      open={open}
      onClose={() => setOpen(false)}
      title={t("release.title")}
      description={
        player
          ? `${player.name ?? t("common.unnamedPlayer")} · ${t("release.description")}`
          : t("release.description")
      }
      width="max-w-[620px]"
      footer={
        <>
          <Button variant="ghost" size="sm" onClick={() => setOpen(false)}>
            {t("common.cancel")}
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={stage}
            disabled={!plan || blocked}
          >
            <UserMinus size={13} />
            {t("release.stage")}
          </Button>
        </>
      }
    >
      {!player ? (
        <div className="flex items-center justify-center py-10 text-[11px] text-[var(--muted)]">
          <Loader2 size={14} className="mr-2 animate-spin" />
          {isLoading ? t("release.loadingPlayer") : t("edit.noPlayer")}
        </div>
      ) : (
        <div className="space-y-4 p-5">
          <div className="flex items-start gap-2 rounded-lg border border-[var(--gold-line)] bg-[var(--gold-soft)] px-3 py-2 text-[10px] leading-relaxed text-[var(--gold)]">
            <AlertTriangle size={13} className="mt-0.5 shrink-0" />
            <span>{t("release.riskWarning")}</span>
          </div>

          <div className="rounded-xl border border-[var(--line)] bg-[var(--surface-2)] px-3 py-2.5">
            <div className="text-[12px] font-semibold text-[var(--ink)]">
              {player.name ?? t("common.unnamedPlayer")}
            </div>
            <div className="mt-0.5 text-[10px] text-[var(--muted)]">
              {t("release.currentClub", {
                club: player.club_name ?? t("common.unknownClub"),
              })}
              {player.natural_positions.length > 0 &&
                ` · ${player.natural_positions.join(" / ")}`}
              {player.age != null &&
                ` · ${t("common.yearsOld", { age: player.age })}`}
            </div>
          </div>

          {(pendingEdit || pendingTransfer || pendingRelease) && (
            <div className="space-y-1 rounded-lg border border-[var(--line)] bg-[var(--surface-2)] px-3 py-2 text-[10px] leading-relaxed text-[var(--muted)]">
              {pendingEdit && <div>{t("transfer.pendingNote")}</div>}
              {pendingTransfer && (
                <div className="text-[var(--danger)]">
                  {t("release.pendingTransfer", {
                    club: pendingTransfer.to_club_name,
                  })}
                </div>
              )}
              {pendingRelease && <div>{t("release.alreadyStaged")}</div>}
            </div>
          )}

          <div>
            <div className="mb-1.5 flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wide text-[var(--muted)]">
              <UserMinus size={11} className="text-[var(--accent)]" />
              {t("release.planTitle")}
            </div>
            {previewing ? (
              <div className="flex items-center justify-center rounded-lg border border-[var(--line)] bg-[var(--surface-2)] px-3 py-4 text-[10px] text-[var(--muted)]">
                <Loader2 size={12} className="mr-2 animate-spin" />
                {t("release.previewing")}
              </div>
            ) : planError ? (
              <div className="rounded-lg border border-[var(--danger-line)] bg-[var(--danger-soft)] px-3 py-2 text-[10px] leading-relaxed text-[var(--danger)]">
                <div>{planError}</div>
                <div className="mt-1 opacity-80">{t("release.guardHint")}</div>
              </div>
            ) : plan ? (
              <div className="rounded-xl border border-[var(--line)] bg-[var(--surface-2)] px-3 py-2.5">
                <div className="flex items-center justify-between gap-3">
                  <div className="min-w-0">
                    <div className="text-[9px] uppercase tracking-wide text-[var(--muted)]">
                      {t("release.from")}
                    </div>
                    <div className="truncate text-[11px] font-medium text-[var(--ink)]">
                      {plan.from_club_name}
                    </div>
                  </div>
                  <div className="shrink-0 text-right text-[10px] text-[var(--muted)]">
                    {t("release.roster", {
                      before: plan.players_before,
                      after: plan.players_after,
                    })}
                  </div>
                </div>
                <ul className="mt-2 space-y-1 text-[10px] text-[var(--muted)]">
                  {plan.lists.length > 1 && (
                    <li>
                      {t("release.extraLists", {
                        count: plan.lists.length - 1,
                      })}
                    </li>
                  )}
                  {plan.captaincy_cleared && (
                    <li>{t("release.captaincyCleared")}</li>
                  )}
                  <li>{t("release.contractKept")}</li>
                  <li>{t("release.selectionWarning")}</li>
                </ul>
              </div>
            ) : null}
          </div>

        </div>
      )}
    </Modal>
  );
}
