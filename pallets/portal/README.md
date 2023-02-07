# XBI portal

XBI portal is the most fully-featured component of XBI. It aims to implement XBI as much as possible through the outlining features below.

## Features

This `pallet` supports most of the req-rep flow of XBI channels. See semantics `here(todo).`

Bridging the edge of the asynchronous XCM world is one of the central tenets of XBI, below outlines those features.

### XBI Channel

`xbi-portal` supports both `receive` and `send` channels. These are backed by the blanket implementation in `crates/sender::frame`
and `crates/receiver::frame`, respectively. They provide an almost complete implementation right out of the box.

There is an instruction handling mechanism that should be provided by each implementer, allowing the parachain to configure what XBI instructions it wants to support.

    E.g.: if you want to support DeFi ops, you will provide some handler in `match xbi.instr { .. }` to support the DeFi operation
    you would like to provide.

We aim to support dynamic ways of handling this routing rather than hardcoded in the future.

### Emitter

Each aspect of the channel can optionally provide an implementation of the `ChannelProgressionEmitter` interface. Since
this is usually unique for an implementer, it is left up to the implementer to provide it. Otherwise, it can be left as
as unit `()`.

### Queue

Another aspect of the channel is that can be handled in many ways:

- Synchronously, `xbi::send()` will invoke `xbi::receive()` on the target, and this will simply try to respond when it sends
  the message back to its sender. Note this implementation does not allow other uses of XBI, namely timeouts, retries, and timestamps.
- A queue will back the asynchronous flow of operations. We provide some semantic HashMap-backed Ringbuffer in `xbi-channel-primitives`.
  This will allow the user to specify how they want the queue to work, whether it's their tolerance to errors, retries, or storage operations.  **MISSSING**: this implementation is yet to be entirely provided, with validation on timeouts,
  conversions from target block time, failing, true queue management

#### Queue implementation: Ringbuffer

We provide an implementation of `xbi-channel-primitives::Queue`, which is backed by a Transient Ringbuffer. This implementation utilizes
some shims from StorageValue to keep the crate from relying on `frame-support`. We then shim these through to `frame-support` on the
implementation of the Ringbuffer.

We want XBI to be used everywhere, so we provide as many interfaces that aren't coupled to `frame` and allow anything to implement the interface:
- a smart contract
- a partial pallet
- a custom xbi portal
- a wrapper
- a client, onchain or offchain

#### Queue management: Dead letters

An essential part of async messaging is the ability to tolerate errors. Messages might fail in many ways, e.g.:
- decoding
- mangling
- network
- funds
- transportation
- timeouts
- cost-tracking

As such, a queue might want to retry managing the message later but not necessarily block picking up new messages.

**MISSING**: Dead letter configuration on the queue (should be configured by the runtime)

#### Queue management: Timeouts

XBI supports user-specified timeouts for each aspect of messaging, including:
- submitted: when a user initially requested a message
- sent: when the sender finally sent the message to the target, this may or may not be within the same block, depending on the sync/async flow
- delivered: when the target successfully received the message
- executed: then the target finished executing the message
- when the sender received the message back

As you can imagine, almost all of these might happen whenever the target is decided, and it's important for an XBI channel to be able to support
asynchronous messaging and synchronous messaging.

On every step, the `timestamp` field will be progressed by the handler of the `XbiMetadata` at that time. This is then utilized by
the queue to validate if it breaches any user-specified timeouts.

**MISSING**: User-specified timeouts are not validated yet, and neither is the differential from the target block number to the current block number.

#### Queue management: Storage

Since the channels have a clear separation of concerns, either:
- in the case of the sender: **building a transport-aware message, applying any costs necessary to the message, and sending the message via the transport mechanism**
- in the case of the receiver: **receive a message, process it, add your timestamps, and send the response back**

Note there is no room here for managing storage items. That is left entirely up to the implementation of the channel. Instead, we provide some storage-backed queue
that will store a generic message's ID and the current signal. The queue then updates the metadata with any results while applying the validation of the timeouts and costs.

Handling validation on costs are currently provided in a limited way. #TODO: Semantically, we should validate on the receiver side if the cost is too much, and then rollback if so.

**MISSING**: Map of XbiId -> State, currently stored just messages and is duplicate storage

## Testing

We have integration testing in `integration-tests`, supported by XCM-emulator.

Currently, the most notable test is `user_pays_for_transact_in_native`, a black box test validating the queue and XBI portal events.
