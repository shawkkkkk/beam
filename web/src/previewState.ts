export const WATCHLIST_KEY = "beam-watchlist";
export const DRAFTS_KEY = "beam-local-drafts";

export type DraftRecord = {
  id: string;
  name: string;
  symbol: string;
  description: string;
  metadataUri: string;
  createdAt: number;
  creatorFeeWallet?: string;
  donationChoice?: "none" | "donate.gg";
  charityName?: string;
  callout?: string;
  livestreamUrl?: string;
};

type StorageLike = Pick<Storage, "getItem" | "setItem">;

const utf8Length = (value: string) => new TextEncoder().encode(value).length;

export function validateCreateV2Metadata(
  name: string,
  symbol: string,
  uri: string,
): string | null {
  if (name.trim().length < 2)
    return "Token name must have at least two characters.";
  if (symbol.trim().length < 2)
    return "Symbol must have at least two characters.";
  if (utf8Length(name) > 32)
    return "Token name exceeds the CreateV2 limit of 32 UTF-8 bytes.";
  if (utf8Length(symbol) > 13)
    return "Symbol exceeds the CreateV2 limit of 13 UTF-8 bytes.";
  if (utf8Length(uri) > 200)
    return "Metadata URI exceeds the CreateV2 limit of 200 UTF-8 bytes.";
  return null;
}

export function validateLocalDraftOptions(
  creatorFeeWallet: string,
  charityName: string,
  callout: string,
  livestreamUrl: string,
): string | null {
  if (
    creatorFeeWallet &&
    !/^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(creatorFeeWallet)
  ) {
    return "Creator fee wallet must look like a Solana address (32–44 base58 characters).";
  }
  if (charityName.length > 80 || callout.length > 180)
    return "Local draft text is too long.";
  if (livestreamUrl) {
    try {
      const url = new URL(livestreamUrl);
      if (url.protocol !== "https:" || livestreamUrl.length > 300)
        throw new Error("invalid");
    } catch {
      return "Livestream URL must be an HTTPS URL under 300 characters.";
    }
  }
  return null;
}

export function getBrowserStorage(): StorageLike | null {
  try {
    return typeof window === "undefined" ? null : window.localStorage;
  } catch {
    return null;
  }
}

export function parseWatchlist(raw: string | null): string[] {
  if (raw === null) return [];
  try {
    const value: unknown = JSON.parse(raw);
    if (
      !Array.isArray(value) ||
      value.length > 1000 ||
      !value.every(
        (id) => typeof id === "string" && id.length > 0 && id.length <= 128,
      )
    )
      return [];
    return [...new Set(value)];
  } catch {
    return [];
  }
}

export function readWatchlist(storage: StorageLike | null): string[] {
  try {
    return parseWatchlist(storage?.getItem(WATCHLIST_KEY) ?? null);
  } catch {
    return [];
  }
}

export function writeWatchlist(
  storage: StorageLike | null,
  ids: string[],
): boolean {
  try {
    if (!storage) return false;
    storage.setItem(WATCHLIST_KEY, JSON.stringify(ids));
    return true;
  } catch {
    return false;
  }
}

function isDraftRecord(value: unknown): value is DraftRecord {
  if (typeof value !== "object" || value === null) return false;
  const draft = value as Partial<DraftRecord>;
  return (
    typeof draft.id === "string" &&
    /^draft-[a-z0-9]+-[0-9]+$/.test(draft.id) &&
    typeof draft.name === "string" &&
    typeof draft.symbol === "string" &&
    typeof draft.description === "string" &&
    draft.description.length <= 180 &&
    typeof draft.metadataUri === "string" &&
    typeof draft.createdAt === "number" &&
    Number.isSafeInteger(draft.createdAt) &&
    draft.createdAt >= 0 &&
    draft.createdAt <= Date.now() + 60_000 &&
    (draft.creatorFeeWallet === undefined ||
      typeof draft.creatorFeeWallet === "string") &&
    (draft.donationChoice === undefined ||
      draft.donationChoice === "none" ||
      draft.donationChoice === "donate.gg") &&
    (draft.charityName === undefined ||
      typeof draft.charityName === "string") &&
    (draft.callout === undefined || typeof draft.callout === "string") &&
    (draft.livestreamUrl === undefined ||
      typeof draft.livestreamUrl === "string") &&
    validateLocalDraftOptions(
      draft.creatorFeeWallet ?? "",
      draft.charityName ?? "",
      draft.callout ?? "",
      draft.livestreamUrl ?? "",
    ) === null &&
    validateCreateV2Metadata(draft.name, draft.symbol, draft.metadataUri) ===
      null
  );
}

export function parseDrafts(raw: string | null): DraftRecord[] {
  if (raw === null) return [];
  try {
    const value: unknown = JSON.parse(raw);
    if (!Array.isArray(value) || value.length > 100) return [];
    const ids = new Set<string>();
    return value.filter((draft): draft is DraftRecord => {
      if (!isDraftRecord(draft) || ids.has(draft.id)) return false;
      ids.add(draft.id);
      return true;
    });
  } catch {
    return [];
  }
}

export function readDrafts(storage: StorageLike | null): DraftRecord[] {
  try {
    return parseDrafts(storage?.getItem(DRAFTS_KEY) ?? null);
  } catch {
    return [];
  }
}

export function writeDrafts(
  storage: StorageLike | null,
  drafts: DraftRecord[],
): boolean {
  try {
    if (!storage) return false;
    storage.setItem(DRAFTS_KEY, JSON.stringify(drafts));
    return true;
  } catch {
    return false;
  }
}

export function nextDraftId(
  used: ReadonlySet<string>,
  timestamp: number,
): string {
  if (!Number.isSafeInteger(timestamp) || timestamp < 0)
    throw new RangeError("Invalid draft time");
  let suffix = 0;
  let id: string;
  do {
    id = `draft-${timestamp.toString(36)}-${suffix++}`;
  } while (used.has(id));
  return id;
}

export function toggleWatchlist(ids: string[], id: string): string[] {
  return ids.includes(id) ? ids.filter((item) => item !== id) : [...ids, id];
}

export function orderNewFeed<T>(drafts: T[], samples: T[]): T[] {
  return [...drafts, ...samples];
}
