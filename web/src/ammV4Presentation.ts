import {
  NO_TRANSACTION_CAPABILITIES,
  type ReadOnlyState,
} from "./indexerAdapter.ts";

/**
 * The current Beam build has an AMM v4 ABI guard, but no executable AMM v4
 * migration or authenticated chain-data capability source. This constant is
 * a statement about this frontend build, not a claim about a deployed program.
 */
export const AMM_V4_BUILD_CAPABILITY = Object.freeze({
  migration: "disabled" as const,
  verifiedLiquidity: false as const,
});

export type AmmV4Presentation = Readonly<{
  /** Every chain-derived phase is explicitly a claim from an unverified API. */
  phase:
    | "unavailable"
    | "on_curve_claim"
    | "curve_complete_claim"
    | "migration_pending_claim"
    | "migration_claimed_unverified";
  graduation: "unavailable" | "disabled" | "unsupported_token_2022";
  /** No authenticated source exists to establish AMM v4 pool identity or reserves. */
  verifiedAmmV4Liquidity: false;
  label: string;
  capabilities: typeof NO_TRANSACTION_CAPABILITIES;
}>;

function presentation(
  phase: AmmV4Presentation["phase"],
  graduation: AmmV4Presentation["graduation"],
  label: string,
): AmmV4Presentation {
  return Object.freeze({
    phase,
    graduation,
    verifiedAmmV4Liquidity: false as const,
    label,
    capabilities: NO_TRANSACTION_CAPABILITIES,
  });
}

/**
 * Project a validated, still unverified indexer candidate for read-only UI.
 * No caller-provided `enabled`, `verified`, `finality`, or pool field can grant
 * a migration or liquidity capability. A future trusted verifier must add a
 * separate authenticated path before a verified-liquidity state can exist.
 */
export function presentAmmV4(
  state: ReadOnlyState<unknown>,
  mint: string,
): AmmV4Presentation {
  if (state.kind !== "unverified_indexer") {
    return presentation(
      "unavailable",
      "unavailable",
      state.kind === "sample"
        ? "Sample preview — curve and AMM v4 state unavailable."
        : "Chain data unavailable — AMM v4 state unavailable.",
    );
  }

  const token = state.page.tokens.find((item) => item.mint === mint);
  const curve = token
    ? state.page.accounts.find((item) => item.address === token.curveAccount)
    : undefined;
  if (!token || !curve) {
    return presentation(
      "unavailable",
      "unavailable",
      "No matching curve account in the unverified indexer page.",
    );
  }

  if (!curve.complete) {
    return presentation(
      "on_curve_claim",
      "disabled",
      "Unverified indexer reports an active curve. AMM v4 migration is disabled in this build.",
    );
  }

  if (token.tokenProgram === "token_2022") {
    return presentation(
      "curve_complete_claim",
      "unsupported_token_2022",
      "Unverified indexer reports a complete curve. AMM v4 does not support Token-2022 graduation.",
    );
  }

  if (curve.migrated && token.lifecycle === "migrated") {
    return presentation(
      "migration_claimed_unverified",
      "disabled",
      "Unverified indexer claims migration. AMM v4 pool identity and liquidity are not verified.",
    );
  }

  return presentation(
    token.lifecycle === "curve_complete"
      ? "migration_pending_claim"
      : "curve_complete_claim",
    "disabled",
    "Unverified indexer reports a complete curve. Migration is pending and disabled in this build.",
  );
}
