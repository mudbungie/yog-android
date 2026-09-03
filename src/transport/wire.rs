//! **Why a gesture did not come back, and which end of the wire is why**
//! (bl-8641, widened for the lost-reply contract in bl-07b1). Split from
//! `transport.rs` when the fourth class arrived: the dialler is one thing and
//! the taxonomy of its failures is another, and only the second is read by
//! callers that never open a socket.

/// The four ways a gesture ends without an answer, told apart by **where the
/// exchange stopped** rather than by what the far end said. A device that
/// decided anything by reading sentences would be one the engine could
/// rewrite by rewording (thrall's bl-916d, adopted in DESIGN §18.5).
///
/// Two questions ride these variants and no third:
///
/// - **Is the channel what failed** — asked by the tool host, which holds one
///   channel and climbs a ladder against it ([`Self::transport`]).
/// - **Was the act written and never answered** — yog REMOTE §3's IN DOUBT,
///   asked by every caller that posts an act ([`Self::in_doubt`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wire {
    /// **The channel failed before the gesture left this end**: a socket that
    /// would not open, a handshake that would not build or would not verify.
    /// Nothing was said, so nothing can have happened — this is the honest
    /// *failed*, and the class a phone meets every time it walks out of range
    /// with nothing in flight.
    Transport(String),
    /// **The gesture was written and nothing answered it** — yog REMOTE §3's
    /// lost reply, and the whole of what this crate calls in doubt: *"A
    /// connection that dies between the engine completing an act and the reply
    /// frame landing tells the client nothing about whether the effect ran,
    /// and nothing on the wire can be added to say it."*
    ///
    /// **The line is where the write ENDS**, and the framing draws it rather
    /// than a guess: an `io::Error` out of a write is that write's bytes not
    /// being accepted, and [`crate::frame`]'s reader takes a length header and
    /// then exactly that many bytes without ever scanning — so a frame that
    /// failed mid-write is a frame no engine decoded. A frame written whole is
    /// the opposite: this end has no way to learn whether it arrived, was
    /// answered, or was answered into a socket that had already gone.
    ///
    /// The engine's own preface is the one exchange *after* the write that is
    /// still definite, so it is [`Self::Unusable`] and not this: yog's
    /// listener writes its version before it reads a request frame
    /// (`wire::hello::admit`), so a peer that never stated one never read the
    /// gesture.
    Lost(String),
    /// **The engine spoke, and what it said is no** — its own refusal,
    /// written as a sentence for an operator. Definite: an act the engine
    /// refused did not run. Whether it is worth asking again is the LEG's
    /// answer and not this one's (`host::serve`, following thrall's bl-916d):
    /// a refusal of this device's *read* is REMOTE §5.1's one-reader guard,
    /// which after a drop names this very device.
    Refused(String),
    /// **An answer this end cannot use**: a frame that is not JSON, a reply of
    /// a kind the gesture does not earn, a version that cannot be spoken to, a
    /// stream ended clean with nothing in it. The engine spoke in every one of
    /// them, and asking again asks the same question and gets the same answer
    /// on every leg, so this class always stops.
    Unusable(String),
}

impl Wire {
    /// The sentence, for the frame that paints it and the caller that wanted
    /// nothing else.
    pub fn sentence(&self) -> String {
        match self {
            Self::Transport(said)
            | Self::Lost(said)
            | Self::Refused(said)
            | Self::Unusable(said) => said.clone(),
        }
    }

    /// Whether the channel is what failed — the one question a caller that can
    /// dial again asks. **Both sides of the write answer yes**: a phone that
    /// loses its network mid-gesture must reconnect exactly as one that lost it
    /// before dialling, and a foot that stopped on a dropped `complete` would
    /// be the wifi-handover defect DESIGN §18.5 exists to prevent. What the
    /// two classes differ about is the ACT, not the channel, and that is
    /// [`Self::in_doubt`]'s question.
    pub fn transport(&self) -> bool {
        matches!(self, Self::Transport(_) | Self::Lost(_))
    }

    /// **Whether an act that failed this way may have run anyway** (yog
    /// REMOTE §3, bl-d1f1). True of exactly one class, and the caller's whole
    /// duty on a `true` is negative: never send the act again. The recovery is
    /// a read of the world, which every caller here already makes on its own
    /// clock.
    ///
    /// An *ask* may ignore this entirely — asking twice is asking once (REMOTE
    /// §9.7), which is why nothing in the read path consults it.
    pub fn in_doubt(&self) -> bool {
        matches!(self, Self::Lost(_))
    }
}

impl From<Wire> for String {
    fn from(wire: Wire) -> Self {
        wire.sentence()
    }
}
