# qtrade2
qtrade2 pulls together Solana vendor rust crates across
- Tooling (anchor)
- DEX (orca, raydium)
- Parsers (vixen)
and offers them in one consistent rust offering.

## The Challenge
Solana has a lot of great rust crates to get you going. The issue is when you want to combine the services from one vendors SDK with that from another. On top of that, getting the correct dependincies in place across the solana-xxx and spl-xxx crates adds another layer of complexity. qtrade2 has done the hard work of methodically layering in many populer Solana vendors SDKs and migrating them to a consistent offering with the base solana-xxx crate set at the 2.x level.

## How it's done
qtrade2 starts with the upcoming version of anchor (0.31) - pulled from anchor's main branch. This is used as the starter set of solana-xxx and spl-xxx crates to use. Then, each vendors SDK is layered in, upgrading and modifying the code as necessary to adhere to the standard set of crate versions set with anchor.
