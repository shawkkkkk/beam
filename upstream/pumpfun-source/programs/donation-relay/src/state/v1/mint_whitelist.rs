use anchor_lang::prelude::*;
// The checked decoder carries this source filename and line 11.
// This restores observed diagnostic data through the actual account macro.
// The earlier historical text is not known.
// Bump is one byte, followed by a Borsh vector of public keys.
// The discriminator is generated from the captured account type name.
// The macro supplies the real checked/unchecked serializers and deserializers.
// No executable instructions or captured data blobs enter this source.
// The allocator and I/O behavior come from the pinned Rust dependencies.

#[account]
pub struct MintWhitelistV1 {pub bump:u8, pub mints:Vec<Pubkey>}
