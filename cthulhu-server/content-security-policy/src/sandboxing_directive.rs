use bitflags::bitflags;

bitflags!{
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SandboxingFlagSet: u32 {
        const SANDBOXED_NAVIGATION_BROWSING_CONTEXT_FLAG = 0x00000001;
        const SANDBOXED_AUXILIARY_NAVIGATION_BROWSING_CONTEXT_FLAG = 0x00000002;
        const SANDBOXED_TOP_LEVEL_NAVIGATION_WITHOUT_USER_ACTIVATION_BROWSING_CONTEXT_FLAG
            = 0x00000004;
        const SANDBOXED_TOP_LEVEL_NAVIGATION_WITH_USER_ACTIVATION_BROWSING_CONTEXT_FLAG
            = 0x00000008;
        const SANDBOXED_PLUGINS_BROWSING_CONTEXT_FLAG = 0x00000010;
        const SANDBOXED_ORIGIN_BROWSING_CONTEXT_FLAG = 0x00000020;
        const SANDBOXED_FORMS_BROWSING_CONTEXT_FLAG = 0x00000040;
        const SANDBOXED_POINTER_LOCK_BROWSING_CONTEXT_FLAG = 0x00000080;
        const SANDBOXED_SCRIPTS_BROWSING_CONTEXT_FLAG = 0x00000100;
        const SANDBOXED_AUTOMATIC_FEATURES_BROWSING_CONTEXT_FLAG = 0x00000200;
        const SANDBOXED_STORAGE_AREA_URLS_FLAG = 0x00000400;
        const SANDBOXED_DOCUMENT_DOMAIN_BROWSING_CONTEXT_FLAG = 0x00000800;
        const SANDBOX_PROPOGATES_TO_AUXILIARY_BROWSING_CONTEXTS_FLAG = 0x00001000;
        const SANDBOXED_MODALS_FLAG = 0x00002000;
        const SANDBOXED_ORIENTATION_LOCK_BROWSING_CONTEXT_FLAG = 0x00004000;
        const SANDBOXED_PRESENTATION_BROWSING_CONTEXT_FLAG = 0x00008000;
    }
}

pub fn parse_a_sandboxing_directive(tokens: &[String]) -> SandboxingFlagSet {
    let mut output = SandboxingFlagSet::all();
    for token in tokens {
        let remove = match &token[..] {
            "allow-popups" =>
                SandboxingFlagSet::SANDBOXED_AUXILIARY_NAVIGATION_BROWSING_CONTEXT_FLAG,
            "allow-top-navigation" =>
                SandboxingFlagSet::SANDBOXED_TOP_LEVEL_NAVIGATION_WITHOUT_USER_ACTIVATION_BROWSING_CONTEXT_FLAG |
                    SandboxingFlagSet::SANDBOXED_TOP_LEVEL_NAVIGATION_WITH_USER_ACTIVATION_BROWSING_CONTEXT_FLAG,
            "allow-top-navigation-by-user-activation" =>
                SandboxingFlagSet::SANDBOXED_TOP_LEVEL_NAVIGATION_WITH_USER_ACTIVATION_BROWSING_CONTEXT_FLAG,
            "allow-same-origin" =>
                SandboxingFlagSet::SANDBOXED_ORIGIN_BROWSING_CONTEXT_FLAG,
            "allow-forms" =>
                SandboxingFlagSet::SANDBOXED_FORMS_BROWSING_CONTEXT_FLAG,
            "allow-pointer-lock" =>
                SandboxingFlagSet::SANDBOXED_POINTER_LOCK_BROWSING_CONTEXT_FLAG,
            "allow-scripts" =>
                SandboxingFlagSet::SANDBOXED_SCRIPTS_BROWSING_CONTEXT_FLAG |
                    SandboxingFlagSet::SANDBOXED_AUTOMATIC_FEATURES_BROWSING_CONTEXT_FLAG,
            "allow-popups-to-escape-sandbox" =>
                SandboxingFlagSet::SANDBOX_PROPOGATES_TO_AUXILIARY_BROWSING_CONTEXTS_FLAG,
            "allow-modals" =>
                SandboxingFlagSet::SANDBOXED_MODALS_FLAG,
            "allow-orientation-lock" =>
                SandboxingFlagSet::SANDBOXED_ORIENTATION_LOCK_BROWSING_CONTEXT_FLAG,
            "allow-presentation" =>
                SandboxingFlagSet::SANDBOXED_PRESENTATION_BROWSING_CONTEXT_FLAG,
            _ =>
                SandboxingFlagSet::empty(),
        };
        output.remove(remove);
    }
    output
}
