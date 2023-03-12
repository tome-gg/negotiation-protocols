use anchor_lang::prelude::*;

declare_id!("5v2iHnzVvmqoYXva1CaDLToUNdwjo1ZHuyMicfokaXBn");

#[program]
pub mod src_alignment_negotiation {
    use super::*;

    pub fn setup_negotation(ctx: Context<SetupNegotiation>, mentor: Pubkey) -> Result<()> {
        print!(
            "🤝 Setting up a negotiation initiated by apprentice `{}` to mentor `{}`",
            ctx.accounts.apprentice.key(),
            mentor.key()
        );
        ctx.accounts
            .negotiation
            .start([ctx.accounts.apprentice.key(), mentor])
    }

    pub fn propose(ctx: Context<Negotiate>, proposal: Proposal) -> Result<()> {
        print!(
            "📝 Sending proposal initiated by `{}`",
            ctx.accounts.player.key()
        );
        require_keys_eq!(
            ctx.accounts.negotiation.current_player(),
            ctx.accounts.player.key(),
            AlignmentError::NotYourTurn
        );

        let mut p = proposal;

        ctx.accounts
            .negotiation
            .negotiate(ctx.accounts.player.key(), &mut p)
    }
}

#[derive(Accounts)]
pub struct SetupNegotiation<'info> {
    #[account(init, payer = apprentice, space = 8 + AlignmentNegotiation::MAXIMUM_SIZE)]
    pub negotiation: Box<Account<'info, AlignmentNegotiation>>,
    #[account(mut)]
    pub apprentice: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AlignmentNegotiation {
    parties: [Pubkey; 2],               // (32 * 2)
    turn: u8,                           // 1
    version: u8,                        // 1
    mentoring_nft: Pubkey,              // 32
    alternatives: Pubkey,               // 32
    term: Pubkey,                       // 32
    parameters: [u8; 32],               // 32
    protocol: Pubkey,                   // 32
    stakes: u64,                        // 8
    term_state: NegotiationState,       // 32 + 1
    protocol_state: NegotiationState,   // 32 + 1
    parameters_state: NegotiationState, // 32 + 1
    stake_state: NegotiationState,      // 32 + 1
    is_complete: bool,                  // 1
}

#[derive(Copy, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum NegotiationState {
    Empty,
    Discussion,
    Proposed { proposer: Pubkey },
    Reviewed { proposee: Pubkey },
    Accepted { proposee: Pubkey },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum NegotiationEvent {
    // Continue discussion
    Discuss,
    // Commit to a new proposal
    Propose,
    // Receive proposal, for review
    Review,
    // Accept proposal
    Accept,
    // Return proposal for discussion
    Decline,
}

pub type AlignmentResult<T> = std::result::Result<T, AlignmentError>;

impl NegotiationState {
    fn display(self) -> String {
        match self {
            NegotiationState::Empty => "empty".to_string(),
            NegotiationState::Discussion => "discussion".to_string(),
            NegotiationState::Proposed { proposer: _ } => "proposed".to_string(),
            NegotiationState::Reviewed { proposee: _ } => "reviewed".to_string(),
            NegotiationState::Accepted { proposee: _ } => "accepted".to_string(),
        }
    }

    // Applies a change on a Pubkey.
    fn update<T: Eq>(
        &self,
        initiator: Pubkey,
        event: NegotiationEvent,
        prev: T,
        new: T,
    ) -> AlignmentResult<NegotiationState> {
        match event {
            NegotiationEvent::Discuss => {
                if prev == new {
                    return Err(AlignmentError::ProposalHasNoChange);
                }

                Ok(NegotiationState::Discussion)
            }
            NegotiationEvent::Propose => {
                if prev == new {
                    return Err(AlignmentError::ProposalHasNoChange);
                }

                if self.is_proposed() || self.is_reviewed() {
                    return Err(AlignmentError::ProposalAlreadySent);
                }

                Ok(NegotiationState::Proposed {
                    proposer: (initiator),
                })
            }
            NegotiationEvent::Review => {
                if self.is_reviewed() {
                    return Err(AlignmentError::ProposalAlreadyReceived);
                }

                Ok(NegotiationState::Reviewed {
                    proposee: (initiator),
                })
            }
            NegotiationEvent::Accept => {
                if self.is_accepted() {
                    return Err(AlignmentError::ProposalAlreadyAccepted);
                }

                Ok(NegotiationState::Accepted {
                    proposee: (initiator),
                })
            }
            NegotiationEvent::Decline => {
                if self.is_accepted() {
                    return Err(AlignmentError::ProposalAlreadyAccepted);
                }

                Ok(NegotiationState::Discussion)
            }
        }
    }

    fn is_proposed(&self) -> bool {
        match self {
            // If the current state is already proposed
            NegotiationState::Proposed { proposer: _ } => {
                return true;
            }
            _ => false,
        }
    }

    fn is_reviewed(&self) -> bool {
        match self {
            // If the current state is already proposed
            NegotiationState::Reviewed { proposee: _ } => {
                return true;
            }
            _ => false,
        }
    }

    fn is_accepted(&self) -> bool {
        match self {
            // If the current state is already proposed
            NegotiationState::Accepted { proposee: _ } => {
                return true;
            }
            _ => false,
        }
    }

    fn is_negotiating(&self) -> bool {
        match self {
            Self::Accepted { proposee: _ } => false,
            _ => true,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Proposal {
    term: Pubkey,         // 32
    parameters: [u8; 32], // 32
    protocol: Pubkey,     // 32
    stakes: u64,          // 8
    events: u16,          // 2
    alt_term: Pubkey,     //  32
    alt_protocol: Pubkey, // 32
}

impl Proposal {
    pub const MAXIMUM_SIZE: usize = 32 + 32 + 32 + 8 + 2 + 32 + 32;

    /* 
    Events
    Events are represented in 16 bits, or two bytes.

    [discuss]   [propose]   [review]    [accept]
    0000        0000        0000        0000

    The bits are grouped into four, which represents the verbs; to discuss, to propose, to review, or to accept.
    
    Each bit in a group represents the object; i.e. term, protocol, parameters, stakes
    
    As such, the following is the interpretation of two bytes.
    
    1000 0100 0010 0001 - Discussing the term, proposing a protocol, reviewing the parameters, accepting the stake.

    */
    pub fn get_term_event(&mut self) -> NegotiationEvent {
        let events = self.events;

        const DISCUSS_TERM: u16 = 0b1000000000000000;
        const PROPOSE_TERM: u16 = 0b0000100000000000;
        const REVIEW_TERM: u16 = 0b0000000010000000;
        const ACCEPT_TERM: u16 = 0b0000000000001000;

        if events & DISCUSS_TERM == DISCUSS_TERM {
            return NegotiationEvent::Discuss;
        }

        if events & PROPOSE_TERM == PROPOSE_TERM {
            return NegotiationEvent::Propose;
        }

        if events & REVIEW_TERM == REVIEW_TERM {
            return NegotiationEvent::Review;
        }

        if events & ACCEPT_TERM == ACCEPT_TERM {
            return NegotiationEvent::Accept;
        }
        NegotiationEvent::Discuss
    }

    pub fn get_protocol_event(&mut self) -> NegotiationEvent {
        let events = self.events;

        const DISCUSS_PROTOCOL: u16 = 0b0100000000000000;
        const PROPOSE_PROTOCOL: u16 = 0b0000010000000000;
        const REVIEW_PROTOCOL: u16 = 0b0000000001000000;
        const ACCEPT_PROTOCOL: u16 = 0b0000000000000100;

        if events & DISCUSS_PROTOCOL == DISCUSS_PROTOCOL {
            return NegotiationEvent::Discuss;
        }

        if events & PROPOSE_PROTOCOL == PROPOSE_PROTOCOL {
            return NegotiationEvent::Propose;
        }

        if events & REVIEW_PROTOCOL == REVIEW_PROTOCOL {
            return NegotiationEvent::Review;
        }

        if events & ACCEPT_PROTOCOL == ACCEPT_PROTOCOL {
            return NegotiationEvent::Accept;
        }
        NegotiationEvent::Discuss
    }

    pub fn get_parameters_event(&mut self) -> NegotiationEvent {
        let events = self.events;

        const DISCUSS_PARAMETERS: u16 = 0b0010000000000000;
        const PROPOSE_PARAMETERS: u16 = 0b0000001000000000;
        const REVIEW_PARAMETERS: u16 = 0b0000000000100000;
        const ACCEPT_PARAMETERS: u16 = 0b0000000000000010;

        if events & DISCUSS_PARAMETERS == DISCUSS_PARAMETERS {
            return NegotiationEvent::Discuss;
        }

        if events & PROPOSE_PARAMETERS == PROPOSE_PARAMETERS {
            return NegotiationEvent::Propose;
        }

        if events & REVIEW_PARAMETERS == REVIEW_PARAMETERS {
            return NegotiationEvent::Review;
        }

        if events & ACCEPT_PARAMETERS == ACCEPT_PARAMETERS {
            return NegotiationEvent::Accept;
        }
        NegotiationEvent::Discuss
    }

    pub fn get_stake_event(&mut self) -> NegotiationEvent {
        let events = self.events;

        const DISCUSS_STAKE: u16 = 0b0001000000000000;
        const PROPOSE_STAKE: u16 = 0b0000000100000000;
        const REVIEW_STAKE: u16 = 0b0000000000010000;
        const ACCEPT_STAKE: u16 = 0b0000000000000001;

        if events & DISCUSS_STAKE == DISCUSS_STAKE {
            return NegotiationEvent::Discuss;
        }

        if events & PROPOSE_STAKE == PROPOSE_STAKE {
            return NegotiationEvent::Propose;
        }

        if events & REVIEW_STAKE == REVIEW_STAKE {
            return NegotiationEvent::Review;
        }

        if events & ACCEPT_STAKE == ACCEPT_STAKE {
            return NegotiationEvent::Accept;
        }
        NegotiationEvent::Discuss
    }
}

#[derive(Accounts)]
pub struct Negotiate<'info> {
    #[account(mut)]
    pub negotiation: Box<Account<'info, AlignmentNegotiation>>,
    pub player: Signer<'info>,
}

#[error_code]
pub enum AlignmentError {
    InvalidNegotiationProtocol,
    InvalidMentoringTerm,
    ProposalHasNoChange,
    ProposalAlreadySent,
    ProposalAlreadyReceived,
    ProposalAlreadyRejected,
    ProposalAlreadyAccepted,
    NegotiationAlreadyOver,
    NotYourTurn,
    NegotiationAlreadyStarted,
}

impl AlignmentNegotiation {
    pub const MAXIMUM_SIZE: usize = (32 * 2) // parties
        + 1 // turn
        + 1 // version
        + 32 // mentoring_nft
        + 32 // alternatives
        + 32 // term
        + 32 // parameters
        + 32 // protocol
        + 8 // stakes
        + (32 + 1) // term_state
        + (32 + 1) // protocol_state
        + (32 + 1) // parameters_state
        + (32 + 1) // stake_state
        + 1; // is_complete

    // Starts the alignment negotiation.
    // This sets the turn counter to 1, and initializes
    // the parties to the negotiation.
    pub fn start(&mut self, parties: [Pubkey; 2]) -> Result<()> {
        require_eq!(self.turn, 0, AlignmentError::NegotiationAlreadyStarted);
        self.parties = parties;
        self.turn = 1;
        self.is_complete = false;
        Ok(())
    }

    // This returns true if and only if any of the states are still active.
    pub fn is_negotiating(&self) -> bool {
        self.term_state.is_negotiating()
            || self.protocol_state.is_negotiating()
            || self.parameters_state.is_negotiating()
            || self.stake_state.is_negotiating()
    }

    pub fn current_player_index(&self) -> usize {
        ((self.turn - 1) % 2) as usize
    }

    pub fn current_player(&self) -> Pubkey {
        self.parties[self.current_player_index()]
    }

    pub fn negotiate(&mut self, initiator: Pubkey, proposal: &mut Proposal) -> Result<()> {
        print!("🤔 Checking if negotiation is over...");
        require!(
            self.is_negotiating(),
            AlignmentError::NegotiationAlreadyOver
        );

        print!("✅ Negotiation still underway.");

        /*
         * Applies the proposal to the current negotiation.
         *
         * This uses the context of the negotiation, the existing state of the
         * negotiation, and this new proposal to determine the new states of the
         * proposal.
         *
         * Returns the four states of the negotiation (term, protocol, parameters, stakes).
         */
        let term_result = self.term_state.update(
            initiator,
            proposal.get_term_event(),
            self.term,
            proposal.term,
        );
        let new_term_state = match term_result {
            Ok(new_state) => new_state,
            Err(reason) => panic!("{}", reason),
        };

        print!("Negotiation term 🔜 {}", new_term_state.display());

        let protocol_result = self.protocol_state.update(
            initiator,
            proposal.get_protocol_event(),
            self.protocol,
            proposal.protocol,
        );
        let new_protocol_state = match protocol_result {
            Ok(new_state) => new_state,
            Err(reason) => panic!("{}", reason),
        };

        print!("Negotiation protocol 🔜 {}", new_protocol_state.display());

        let parameters_result = self.parameters_state.update(
            initiator,
            proposal.get_parameters_event(),
            self.parameters,
            proposal.parameters,
        );
        let new_parameters_state = match parameters_result {
            Ok(new_state) => new_state,
            Err(reason) => panic!("{}", reason),
        };

        print!(
            "Negotiation parameters 🔜 {}",
            new_parameters_state.display()
        );

        let stake_result = self.stake_state.update(
            initiator,
            proposal.get_stake_event(),
            self.stakes,
            proposal.stakes,
        );
        let new_stake_state = match stake_result {
            Ok(new_state) => new_state,
            Err(reason) => panic!("{}", reason),
        };

        print!("Negotiation stake 🔜 {}", new_stake_state.display());

        print!("📝 Updating negotiation values...");

        // Update negotiation
        self.term = proposal.term;
        self.protocol = proposal.protocol;
        self.parameters = proposal.parameters;
        self.stakes = proposal.stakes;

        print!("📝 Updating negotiation states...");

        // Update negotiation states
        self.term_state = new_term_state;
        self.protocol_state = new_protocol_state;
        self.parameters_state = new_parameters_state;
        self.stake_state = new_stake_state;

        print!("🤔 Checking if negotiation is finished...");
        self.update_state();

        if self.is_complete == false {
            print!("⏳️ Negotiation not yet finished.");
            self.turn += 1;
        } else {
            print!("⌛️ Negotiation complete.");
        }

        Ok(())
    }

    fn update_state(&mut self) {
        self.is_complete = self.is_negotiating() == false;
    }
}
