use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum StreamingProtocol {
    Https,
    Dash,
    Hls,
}

#[derive(Debug, Clone, Copy)]
pub struct BasePoTokenPolicy {
    pub required: bool,
    pub recommended: bool,
    pub not_required_for_premium: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct GvsPoTokenPolicy {
    pub base: BasePoTokenPolicy,
    pub not_required_with_player_token: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerPoTokenPolicy {
    pub required: bool,
    pub recommended: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SubsPoTokenPolicy {
    pub required: bool,
}

// The top-level container
#[derive(Debug)]
pub struct WebPoTokenPolicies {
    pub gvs_po_token_policy: HashMap<StreamingProtocol, GvsPoTokenPolicy>,
    pub player_po_token_policy: PlayerPoTokenPolicy,
    pub subs_po_token_policy: SubsPoTokenPolicy,
}

impl WebPoTokenPolicies {
    pub fn new() -> Self {
        let mut gvs = HashMap::new();

        let base_required = BasePoTokenPolicy {
            required: true,
            recommended: true,
            not_required_for_premium: true,
        };

        gvs.insert(
            StreamingProtocol::Https,
            GvsPoTokenPolicy {
                base: base_required,
                not_required_with_player_token: false,
            },
        );

        gvs.insert(
            StreamingProtocol::Dash,
            GvsPoTokenPolicy {
                base: base_required,
                not_required_with_player_token: false,
            },
        );

        gvs.insert(
            StreamingProtocol::Hls,
            GvsPoTokenPolicy {
                base: BasePoTokenPolicy {
                    required: false,
                    recommended: true,
                    not_required_for_premium: false,
                },
                not_required_with_player_token: false,
            },
        );

        Self {
            gvs_po_token_policy: gvs,
            player_po_token_policy: PlayerPoTokenPolicy {
                required: false,
                recommended: false,
            },
            subs_po_token_policy: SubsPoTokenPolicy { required: false },
        }
    }
}
