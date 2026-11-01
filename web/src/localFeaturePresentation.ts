import type { DraftRecord } from "./previewState";

type LocalOptions = Pick<
  DraftRecord,
  | "creatorFeeWallet"
  | "donationChoice"
  | "charityName"
  | "callout"
  | "livestreamUrl"
>;

export type LocalFeaturePlan = Readonly<{
  kind: "creator_wallet" | "charity" | "callout" | "livestream";
  label: string;
  value: string;
  source: "local_draft";
  availability: "unavailable";
  reason: string;
}>;

/** Display only local draft intentions. No backend capability is inferred. */
export function localFeaturePlans(
  draft: LocalOptions,
): readonly LocalFeaturePlan[] {
  const plans: LocalFeaturePlan[] = [];
  const add = (
    kind: LocalFeaturePlan["kind"],
    label: string,
    value: string,
    reason: string,
  ) => {
    plans.push(
      Object.freeze({
        kind,
        label,
        value,
        source: "local_draft",
        availability: "unavailable",
        reason,
      }),
    );
  };
  if (draft.creatorFeeWallet?.trim()) {
    add(
      "creator_wallet",
      "Creator fee wallet plan",
      draft.creatorFeeWallet.trim(),
      "No wallet has been selected on chain.",
    );
  }
  if (draft.donationChoice === "donate.gg") {
    add(
      "charity",
      "Donate.gg interest",
      draft.charityName?.trim() || "Charity not selected",
      "No charity or donation provider is verified or connected.",
    );
  }
  if (draft.callout?.trim()) {
    add(
      "callout",
      "Callout draft",
      draft.callout.trim(),
      "Not submitted, moderated, or displayed.",
    );
  }
  if (draft.livestreamUrl?.trim()) {
    add(
      "livestream",
      "Stream URL note",
      draft.livestreamUrl.trim(),
      "No broadcast, media provider, or playback is available.",
    );
  }
  return Object.freeze(plans);
}
