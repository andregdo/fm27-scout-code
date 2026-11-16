import { useEffect, useRef, useState } from "react";
import {
  AlertTriangle,
  ArrowRightLeft,
  Check,
  Loader2,
  Search,
  ShieldCheck,
} from "lucide-react";
import { pickEditOutputFile, previewTransfer, transferPlayer } from "@/lib/api";
import { useOpenSave } from "@/lib/openSave";
import { usePlayerDetail, useTransferClubs } from "@/lib/query";
import type { ClubRow, TransferChange, TransferResult } from "@/lib/types";
import { cn, suggestedTransferredSavePath } from "@/lib/utils";
import { useI18n } from "@/lib/useI18n";
import { useAppStore } from "@/store/app";
import { Button, Input, Modal } from "./ui";

export function TransferDialog() {
  const { t } = useI18n();
  const open = useAppStore((state) => state.transferOpen);
  const setOpen = useAppStore((state) => state.setTransferOpen);
  const saveId = useAppStore((state) => state.saveId);
  const summary = useAppStore((state) => state.summary);
  const selectedUid = useAppStore((state) => state.selectedUid);
  const pendingEdits = useAppStore((state) => state.pendingEdits);
  const pushToast = useAppStore((state) => state.pushToast);
  const { openPath } = useOpenSave();
  const { data: player, isLoading } = usePlayerDetail(open ? selectedUid : null);
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [destination, setDestination] = useState<ClubRow | null>(null);
  const [plan, setPlan] = useState<TransferChange | null>(null);
  const [previewing, setPreviewing] = useState(false);
  const [planError, setPlanError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<TransferResult | null>(null);
  const requestId = useRef(0);

  const clubsQuery = useTransferClubs(
    debouncedSearch,
    player?.gender ?? null,
    open && player != null,
  );
  const clubs = (clubsQuery.data?.rows ?? []).filter(
    (club) => club.uid !== player?.club_uid,
  );
  const pendingEdit = player ? pendingEdits[player.uid] : undefined;

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedSearch(search), 250);
    return () => window.clearTimeout(timer);
  }, [search]);

  // A new player or a reopened dialog starts a fresh plan.
  useEffect(() => {
    if (!open) return;
    requestId.current += 1;
    setSearch("");
    setDebouncedSearch("");
    setDestination(null);
    setPlan(null);
    setPreviewing(false);
    setPlanError(null);
    setBusy(false);
    setError(null);
    setResult(null);
  }, [open, selectedUid]);

  const chooseDestination = async (club: ClubRow) => {
    if (saveId == null || !player) return;
    setDestination(club);
    setPlan(null);
    setPlanError(null);
    setError(null);
    setPreviewing(true);
    const request = ++requestId.current;
    try {
      const preview = await previewTransfer(saveId, player.uid, club.uid);
      if (request === requestId.current) setPlan(preview);
    } catch (previewError) {
      if (request === requestId.current) setPlanError(String(previewError));
    } finally {
      if (request === requestId.current) setPreviewing(false);
    }
  };

  const save = async () => {
    if (saveId == null || !player || !destination || !plan) return;
    if (plan.to_club_uid !== destination.uid) return;
    const output = await pickEditOutputFile(
      suggestedTransferredSavePath(summary?.path, summary?.save_name),
    );
    if (!output) return;
    setBusy(true);
    setError(null);
    try {
      const written = await transferPlayer(saveId, player.uid, destination.uid, output);
      setResult(written);
      pushToast({
        title: t("transfer.done"),
        description: written.output,
        tone: "success",
      });
    } catch (writeError) {
      const message = String(writeError);
      setError(message);
      pushToast({ title: t("transfer.failed"), description: message, tone: "error" });
    } finally {
      setBusy(false);
    }
  };

  const close = () => {
    if (!busy) setOpen(false);
  };

  const contractRestarts = plan?.changes.some(
    (change) => change.field === "contract.start",
  );

  return (
    <Modal
      open={open}
      onClose={close}
      title={t("transfer.title")}
      description={
        player
          ? `${player.name ?? t("common.unnamedPlayer")} · ${t("transfer.description")}`
          : t("transfer.description")
      }
      width="max-w-[720px]"
      footer={
        result ? (
          <>
            <Button variant="ghost" size="sm" onClick={close}>
              {t("common.close")}
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={() => {
                setOpen(false);
                void openPath(result.output, true);
              }}
            >
              {t("transfer.open")}
            </Button>
          </>
        ) : (
          <>
            <Button variant="ghost" size="sm" onClick={close} disabled={busy}>
              {t("common.cancel")}
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={() => void save()}
              disabled={busy || !plan || plan.to_club_uid !== destination?.uid}
            >
              {busy ? (
                <Loader2 size={13} className="animate-spin" />
              ) : (
                <ArrowRightLeft size={13} />
              )}
              {busy ? t("transfer.saving") : t("transfer.save")}
            </Button>
          </>
        )
      }
    >
      {result ? (
        <div className="flex flex-col items-center gap-3 px-6 py-10 text-center">
          <div className="flex h-12 w-12 items-center justify-center rounded-2xl border border-[var(--accent-line)] bg-[var(--accent-soft)] text-[var(--accent-strong)]">
            <ShieldCheck size={22} />
          </div>
          <div className="text-sm font-semibold text-[var(--ink)]">
            {t("transfer.successTitle")}
          </div>
          <div className="max-w-full break-all text-[11px] text-[var(--muted)]">
            {result.output}
          </div>
          <div className="text-[10px] text-[var(--muted)]">
            {t("transfer.successSummary", {
              from: result.transfer.from_club_name,
              to: result.transfer.to_club_name,
            })}
          </div>
          <div className="max-w-md text-[10px] leading-relaxed text-[var(--muted)]">
            {t("transfer.successNote")}
          </div>
        </div>
      ) : !player ? (
        <div className="flex items-center justify-center py-10 text-[11px] text-[var(--muted)]">
          <Loader2 size={14} className="mr-2 animate-spin" />
          {isLoading ? t("transfer.loadingPlayer") : t("edit.noPlayer")}
        </div>
      ) : (
        <div className="space-y-4 p-5">
          {error && (
            <div className="rounded-lg border border-[var(--danger-line)] bg-[var(--danger-soft)] px-3 py-2 text-[11px] text-[var(--danger)]">
              {error}
            </div>
          )}
          <div className="flex items-start gap-2 rounded-lg border border-[var(--gold-line)] bg-[var(--gold-soft)] px-3 py-2 text-[10px] leading-relaxed text-[var(--gold)]">
            <AlertTriangle size={13} className="mt-0.5 shrink-0" />
            <span>{t("transfer.riskWarning")}</span>
          </div>

          <div className="rounded-xl border border-[var(--line)] bg-[var(--surface-2)] px-3 py-2.5">
            <div className="text-[12px] font-semibold text-[var(--ink)]">
              {player.name ?? t("common.unnamedPlayer")}
            </div>
            <div className="mt-0.5 text-[10px] text-[var(--muted)]">
              {t("transfer.currentClub", {
                club: player.club_name ?? t("common.unknownClub"),
              })}
              {player.natural_positions.length > 0 &&
                ` · ${player.natural_positions.join(" / ")}`}
              {player.age != null &&
                ` · ${t("common.yearsOld", { age: player.age })}`}
            </div>
          </div>

          {pendingEdit && (
            <div className="rounded-lg border border-[var(--line)] bg-[var(--surface-2)] px-3 py-2 text-[10px] leading-relaxed text-[var(--muted)]">
              {t("transfer.pendingNote")}
            </div>
          )}

          <div>
            <div className="mb-1.5 text-[10px] font-semibold uppercase tracking-wide text-[var(--muted)]">
              {t("transfer.destination")}
            </div>
            <div className="relative">
              <Search
                size={12}
                className="pointer-events-none absolute left-2.5 top-2.5 text-[var(--muted)]"
              />
              <Input
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder={t("transfer.searchClubs")}
                className="pl-7"
                autoFocus
              />
            </div>
            <div className="thin-scroll mt-2 max-h-56 space-y-1 overflow-y-auto">
              {clubsQuery.isFetching && clubs.length === 0 ? (
                <div className="flex items-center justify-center py-6 text-[10px] text-[var(--muted)]">
                  <Loader2 size={12} className="mr-2 animate-spin" />
                  {t("transfer.loadingClubs")}
                </div>
              ) : clubs.length === 0 ? (
                <div className="py-6 text-center text-[10px] text-[var(--muted)]">
                  {t("transfer.noClubs")}
                </div>
              ) : (
                clubs.map((club) => {
                  const selected = destination?.uid === club.uid;
                  return (
                    <button
                      key={club.uid}
                      type="button"
                      onClick={() => void chooseDestination(club)}
                      className={cn(
                        "flex w-full items-center justify-between gap-3 rounded-lg border px-2.5 py-2 text-left transition-colors",
                        selected
                          ? "border-[var(--accent)] bg-[var(--accent-soft)]"
                          : "border-[var(--line)] bg-[var(--surface)] hover:bg-[var(--surface-2)]",
                      )}
                    >
                      <div className="min-w-0">
                        <div className="truncate text-[11px] font-medium text-[var(--ink)]">
                          {club.name}
                        </div>
                        <div className="truncate text-[9px] text-[var(--muted)]">
                          {[club.nation_name, club.league_name]
                            .filter((value): value is string => Boolean(value))
                            .join(" · ")}
                          {club.player_count > 0 &&
                            ` · ${t("common.playersCount", {
                              count: club.player_count,
                            })}`}
                        </div>
                      </div>
                      {selected && (
                        <Check size={12} className="shrink-0 text-[var(--accent)]" />
                      )}
                    </button>
                  );
                })
              )}
            </div>
          </div>

          <div>
            <div className="mb-1.5 flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wide text-[var(--muted)]">
              <ArrowRightLeft size={11} className="text-[var(--accent)]" />
              {t("transfer.planTitle")}
            </div>
            {!destination ? (
              <div className="rounded-lg border border-dashed border-[var(--line)] px-3 py-4 text-center text-[10px] text-[var(--muted)]">
                {t("transfer.chooseHint")}
              </div>
            ) : previewing ? (
              <div className="flex items-center justify-center rounded-lg border border-[var(--line)] bg-[var(--surface-2)] px-3 py-4 text-[10px] text-[var(--muted)]">
                <Loader2 size={12} className="mr-2 animate-spin" />
                {t("transfer.previewing")}
              </div>
            ) : planError ? (
              <div className="rounded-lg border border-[var(--danger-line)] bg-[var(--danger-soft)] px-3 py-2 text-[10px] leading-relaxed text-[var(--danger)]">
                <div>{planError}</div>
                <div className="mt-1 opacity-80">{t("transfer.guardHint")}</div>
              </div>
            ) : plan ? (
              <div className="rounded-xl border border-[var(--line)] bg-[var(--surface-2)] px-3 py-2.5">
                <div className="grid grid-cols-2 gap-2">
                  <div className="min-w-0">
                    <div className="text-[9px] uppercase tracking-wide text-[var(--muted)]">
                      {t("transfer.from")}
                    </div>
                    <div className="truncate text-[11px] font-medium text-[var(--ink)]">
                      {plan.from_club_name}
                    </div>
                    <div className="text-[9px] text-[var(--muted)]">
                      {t("transfer.roster", {
                        before: plan.from_players_before,
                        after: plan.from_players_after,
                      })}
                    </div>
                  </div>
                  <div className="min-w-0">
                    <div className="text-[9px] uppercase tracking-wide text-[var(--muted)]">
                      {t("transfer.to")}
                    </div>
                    <div className="truncate text-[11px] font-medium text-[var(--ink)]">
                      {plan.to_club_name}
                    </div>
                    <div className="text-[9px] text-[var(--muted)]">
                      {t("transfer.roster", {
                        before: plan.to_players_before,
                        after: plan.to_players_after,
                      })}
                    </div>
                  </div>
                </div>
                <ul className="mt-2 space-y-1 text-[10px] text-[var(--muted)]">
                  {contractRestarts && (
                    <li>
                      {t("transfer.contractRestart", {
                        date: summary?.game_date ?? "—",
                      })}
                    </li>
                  )}
                  {plan.changes.some((change) => change.field === "contract.listing") && (
                    <li>{t("transfer.listingCleared")}</li>
                  )}
                  {plan.captaincy_cleared && <li>{t("transfer.captaincyCleared")}</li>}
                  {plan.squad_number_before != null && (
                    <li>
                      {t("transfer.squadNumberCleared", {
                        number: plan.squad_number_before,
                      })}
                    </li>
                  )}
                  <li>{t("transfer.noFee")}</li>
                </ul>
              </div>
            ) : null}
          </div>
        </div>
      )}
    </Modal>
  );
}
