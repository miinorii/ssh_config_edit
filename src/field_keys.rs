//! Typed representation of ssh_config(5) keywords.
//!
//! Keyword list derived from the OpenSSH 9.9p2 ssh_config(5) manual page:
//! https://man7.org/linux/man-pages/man5/ssh_config.5.html
//!
//! Per ssh_config(5), known keywords are case-insensitive.
//!
//! Disclaimer: due to the sheer number of parameters, file generation was automated using AI.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldKey {
    Host,
    Match,
    AddKeysToAgent,
    AddressFamily,
    BatchMode,
    BindAddress,
    BindInterface,
    CanonicalDomains,
    CanonicalizeFallbackLocal,
    CanonicalizeHostname,
    CanonicalizeMaxDots,
    CanonicalizePermittedCNAMEs,
    CASignatureAlgorithms,
    CertificateFile,
    ChannelTimeout,
    CheckHostIP,
    Ciphers,
    ClearAllForwardings,
    Compression,
    ConnectionAttempts,
    ConnectTimeout,
    ControlMaster,
    ControlPath,
    ControlPersist,
    DynamicForward,
    EnableEscapeCommandline,
    EnableSSHKeysign,
    EscapeChar,
    ExitOnForwardFailure,
    FingerprintHash,
    ForkAfterAuthentication,
    ForwardAgent,
    ForwardX11,
    ForwardX11Timeout,
    ForwardX11Trusted,
    GatewayPorts,
    GlobalKnownHostsFile,
    GSSAPIAuthentication,
    GSSAPIDelegateCredentials,
    HashKnownHosts,
    HostbasedAcceptedAlgorithms,
    HostbasedAuthentication,
    HostKeyAlgorithms,
    HostKeyAlias,
    Hostname,
    IdentitiesOnly,
    IdentityAgent,
    IdentityFile,
    IgnoreUnknown,
    Include,
    IPQoS,
    KbdInteractiveAuthentication,
    KbdInteractiveDevices,
    KexAlgorithms,
    KnownHostsCommand,
    LocalCommand,
    LocalForward,
    LogLevel,
    LogVerbose,
    MACs,
    NoHostAuthenticationForLocalhost,
    NumberOfPasswordPrompts,
    ObscureKeystrokeTiming,
    PasswordAuthentication,
    PermitLocalCommand,
    PermitRemoteOpen,
    PKCS11Provider,
    Port,
    PreferredAuthentications,
    ProxyCommand,
    ProxyJump,
    ProxyUseFdpass,
    PubkeyAcceptedAlgorithms,
    PubkeyAuthentication,
    RekeyLimit,
    RemoteCommand,
    RemoteForward,
    RequestTTY,
    RequiredRSASize,
    RevokedHostKeys,
    SecurityKeyProvider,
    SendEnv,
    ServerAliveCountMax,
    ServerAliveInterval,
    SessionType,
    SetEnv,
    StdinNull,
    StreamLocalBindMask,
    StreamLocalBindUnlink,
    StrictHostKeyChecking,
    SyslogFacility,
    TCPKeepAlive,
    Tag,
    Tunnel,
    TunnelDevice,
    UpdateHostKeys,
    User,
    UserKnownHostsFile,
    VerifyHostKeyDNS,
    VisualHostKey,
    XAuthLocation,
    Other(String),
}

impl FieldKey {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "host" => FieldKey::Host,
            "match" => FieldKey::Match,
            "addkeystoagent" => FieldKey::AddKeysToAgent,
            "addressfamily" => FieldKey::AddressFamily,
            "batchmode" => FieldKey::BatchMode,
            "bindaddress" => FieldKey::BindAddress,
            "bindinterface" => FieldKey::BindInterface,
            "canonicaldomains" => FieldKey::CanonicalDomains,
            "canonicalizefallbacklocal" => FieldKey::CanonicalizeFallbackLocal,
            "canonicalizehostname" => FieldKey::CanonicalizeHostname,
            "canonicalizemaxdots" => FieldKey::CanonicalizeMaxDots,
            "canonicalizepermittedcnames" => FieldKey::CanonicalizePermittedCNAMEs,
            "casignaturealgorithms" => FieldKey::CASignatureAlgorithms,
            "certificatefile" => FieldKey::CertificateFile,
            "channeltimeout" => FieldKey::ChannelTimeout,
            "checkhostip" => FieldKey::CheckHostIP,
            "ciphers" => FieldKey::Ciphers,
            "clearallforwardings" => FieldKey::ClearAllForwardings,
            "compression" => FieldKey::Compression,
            "connectionattempts" => FieldKey::ConnectionAttempts,
            "connecttimeout" => FieldKey::ConnectTimeout,
            "controlmaster" => FieldKey::ControlMaster,
            "controlpath" => FieldKey::ControlPath,
            "controlpersist" => FieldKey::ControlPersist,
            "dynamicforward" => FieldKey::DynamicForward,
            "enableescapecommandline" => FieldKey::EnableEscapeCommandline,
            "enablesshkeysign" => FieldKey::EnableSSHKeysign,
            "escapechar" => FieldKey::EscapeChar,
            "exitonforwardfailure" => FieldKey::ExitOnForwardFailure,
            "fingerprinthash" => FieldKey::FingerprintHash,
            "forkafterauthentication" => FieldKey::ForkAfterAuthentication,
            "forwardagent" => FieldKey::ForwardAgent,
            "forwardx11" => FieldKey::ForwardX11,
            "forwardx11timeout" => FieldKey::ForwardX11Timeout,
            "forwardx11trusted" => FieldKey::ForwardX11Trusted,
            "gatewayports" => FieldKey::GatewayPorts,
            "globalknownhostsfile" => FieldKey::GlobalKnownHostsFile,
            "gssapiauthentication" => FieldKey::GSSAPIAuthentication,
            "gssapidelegatecredentials" => FieldKey::GSSAPIDelegateCredentials,
            "hashknownhosts" => FieldKey::HashKnownHosts,
            "hostbasedacceptedalgorithms" => FieldKey::HostbasedAcceptedAlgorithms,
            "hostbasedauthentication" => FieldKey::HostbasedAuthentication,
            "hostkeyalgorithms" => FieldKey::HostKeyAlgorithms,
            "hostkeyalias" => FieldKey::HostKeyAlias,
            "hostname" => FieldKey::Hostname,
            "identitiesonly" => FieldKey::IdentitiesOnly,
            "identityagent" => FieldKey::IdentityAgent,
            "identityfile" => FieldKey::IdentityFile,
            "ignoreunknown" => FieldKey::IgnoreUnknown,
            "include" => FieldKey::Include,
            "ipqos" => FieldKey::IPQoS,
            "kbdinteractiveauthentication" => FieldKey::KbdInteractiveAuthentication,
            "kbdinteractivedevices" => FieldKey::KbdInteractiveDevices,
            "kexalgorithms" => FieldKey::KexAlgorithms,
            "knownhostscommand" => FieldKey::KnownHostsCommand,
            "localcommand" => FieldKey::LocalCommand,
            "localforward" => FieldKey::LocalForward,
            "loglevel" => FieldKey::LogLevel,
            "logverbose" => FieldKey::LogVerbose,
            "macs" => FieldKey::MACs,
            "nohostauthenticationforlocalhost" => FieldKey::NoHostAuthenticationForLocalhost,
            "numberofpasswordprompts" => FieldKey::NumberOfPasswordPrompts,
            "obscurekeystroketiming" => FieldKey::ObscureKeystrokeTiming,
            "passwordauthentication" => FieldKey::PasswordAuthentication,
            "permitlocalcommand" => FieldKey::PermitLocalCommand,
            "permitremoteopen" => FieldKey::PermitRemoteOpen,
            "pkcs11provider" => FieldKey::PKCS11Provider,
            "port" => FieldKey::Port,
            "preferredauthentications" => FieldKey::PreferredAuthentications,
            "proxycommand" => FieldKey::ProxyCommand,
            "proxyjump" => FieldKey::ProxyJump,
            "proxyusefdpass" => FieldKey::ProxyUseFdpass,
            "pubkeyacceptedalgorithms" => FieldKey::PubkeyAcceptedAlgorithms,
            "pubkeyauthentication" => FieldKey::PubkeyAuthentication,
            "rekeylimit" => FieldKey::RekeyLimit,
            "remotecommand" => FieldKey::RemoteCommand,
            "remoteforward" => FieldKey::RemoteForward,
            "requesttty" => FieldKey::RequestTTY,
            "requiredrsasize" => FieldKey::RequiredRSASize,
            "revokedhostkeys" => FieldKey::RevokedHostKeys,
            "securitykeyprovider" => FieldKey::SecurityKeyProvider,
            "sendenv" => FieldKey::SendEnv,
            "serveralivecountmax" => FieldKey::ServerAliveCountMax,
            "serveraliveinterval" => FieldKey::ServerAliveInterval,
            "sessiontype" => FieldKey::SessionType,
            "setenv" => FieldKey::SetEnv,
            "stdinnull" => FieldKey::StdinNull,
            "streamlocalbindmask" => FieldKey::StreamLocalBindMask,
            "streamlocalbindunlink" => FieldKey::StreamLocalBindUnlink,
            "stricthostkeychecking" => FieldKey::StrictHostKeyChecking,
            "syslogfacility" => FieldKey::SyslogFacility,
            "tcpkeepalive" => FieldKey::TCPKeepAlive,
            "tag" => FieldKey::Tag,
            "tunnel" => FieldKey::Tunnel,
            "tunneldevice" => FieldKey::TunnelDevice,
            "updatehostkeys" => FieldKey::UpdateHostKeys,
            "user" => FieldKey::User,
            "userknownhostsfile" => FieldKey::UserKnownHostsFile,
            "verifyhostkeydns" => FieldKey::VerifyHostKeyDNS,
            "visualhostkey" => FieldKey::VisualHostKey,
            "xauthlocation" => FieldKey::XAuthLocation,
            _ => FieldKey::Other(s.to_string()),
        }
    }

    /// Whether this directive accumulates across matching keys (each
    /// occurrence is appended) rather than following first-match-wins.
    ///
    /// Derived from the `dump_cfg_strarray` and `dump_cfg_forwards` handling
    /// in OpenSSH's readconf.c
    pub fn is_cumulative(&self) -> bool {
        matches!(
            self,
            FieldKey::IdentityFile
                | FieldKey::CertificateFile
                | FieldKey::SendEnv
                | FieldKey::SetEnv
                | FieldKey::LocalForward
                | FieldKey::RemoteForward
                | FieldKey::DynamicForward
        )
    }

    /// Whether this keyword is a selector rather than a setting.
    pub fn is_selector(&self) -> bool {
        matches!(self, FieldKey::Host | FieldKey::Match)
    }

    /// The canonical spelling of the keyword (as documented in ssh_config(5)).
    pub fn as_canonical_str(&self) -> &str {
        match self {
            FieldKey::Host => "Host",
            FieldKey::Match => "Match",
            FieldKey::AddKeysToAgent => "AddKeysToAgent",
            FieldKey::AddressFamily => "AddressFamily",
            FieldKey::BatchMode => "BatchMode",
            FieldKey::BindAddress => "BindAddress",
            FieldKey::BindInterface => "BindInterface",
            FieldKey::CanonicalDomains => "CanonicalDomains",
            FieldKey::CanonicalizeFallbackLocal => "CanonicalizeFallbackLocal",
            FieldKey::CanonicalizeHostname => "CanonicalizeHostname",
            FieldKey::CanonicalizeMaxDots => "CanonicalizeMaxDots",
            FieldKey::CanonicalizePermittedCNAMEs => "CanonicalizePermittedCNAMEs",
            FieldKey::CASignatureAlgorithms => "CASignatureAlgorithms",
            FieldKey::CertificateFile => "CertificateFile",
            FieldKey::ChannelTimeout => "ChannelTimeout",
            FieldKey::CheckHostIP => "CheckHostIP",
            FieldKey::Ciphers => "Ciphers",
            FieldKey::ClearAllForwardings => "ClearAllForwardings",
            FieldKey::Compression => "Compression",
            FieldKey::ConnectionAttempts => "ConnectionAttempts",
            FieldKey::ConnectTimeout => "ConnectTimeout",
            FieldKey::ControlMaster => "ControlMaster",
            FieldKey::ControlPath => "ControlPath",
            FieldKey::ControlPersist => "ControlPersist",
            FieldKey::DynamicForward => "DynamicForward",
            FieldKey::EnableEscapeCommandline => "EnableEscapeCommandline",
            FieldKey::EnableSSHKeysign => "EnableSSHKeysign",
            FieldKey::EscapeChar => "EscapeChar",
            FieldKey::ExitOnForwardFailure => "ExitOnForwardFailure",
            FieldKey::FingerprintHash => "FingerprintHash",
            FieldKey::ForkAfterAuthentication => "ForkAfterAuthentication",
            FieldKey::ForwardAgent => "ForwardAgent",
            FieldKey::ForwardX11 => "ForwardX11",
            FieldKey::ForwardX11Timeout => "ForwardX11Timeout",
            FieldKey::ForwardX11Trusted => "ForwardX11Trusted",
            FieldKey::GatewayPorts => "GatewayPorts",
            FieldKey::GlobalKnownHostsFile => "GlobalKnownHostsFile",
            FieldKey::GSSAPIAuthentication => "GSSAPIAuthentication",
            FieldKey::GSSAPIDelegateCredentials => "GSSAPIDelegateCredentials",
            FieldKey::HashKnownHosts => "HashKnownHosts",
            FieldKey::HostbasedAcceptedAlgorithms => "HostbasedAcceptedAlgorithms",
            FieldKey::HostbasedAuthentication => "HostbasedAuthentication",
            FieldKey::HostKeyAlgorithms => "HostKeyAlgorithms",
            FieldKey::HostKeyAlias => "HostKeyAlias",
            FieldKey::Hostname => "Hostname",
            FieldKey::IdentitiesOnly => "IdentitiesOnly",
            FieldKey::IdentityAgent => "IdentityAgent",
            FieldKey::IdentityFile => "IdentityFile",
            FieldKey::IgnoreUnknown => "IgnoreUnknown",
            FieldKey::Include => "Include",
            FieldKey::IPQoS => "IPQoS",
            FieldKey::KbdInteractiveAuthentication => "KbdInteractiveAuthentication",
            FieldKey::KbdInteractiveDevices => "KbdInteractiveDevices",
            FieldKey::KexAlgorithms => "KexAlgorithms",
            FieldKey::KnownHostsCommand => "KnownHostsCommand",
            FieldKey::LocalCommand => "LocalCommand",
            FieldKey::LocalForward => "LocalForward",
            FieldKey::LogLevel => "LogLevel",
            FieldKey::LogVerbose => "LogVerbose",
            FieldKey::MACs => "MACs",
            FieldKey::NoHostAuthenticationForLocalhost => "NoHostAuthenticationForLocalhost",
            FieldKey::NumberOfPasswordPrompts => "NumberOfPasswordPrompts",
            FieldKey::ObscureKeystrokeTiming => "ObscureKeystrokeTiming",
            FieldKey::PasswordAuthentication => "PasswordAuthentication",
            FieldKey::PermitLocalCommand => "PermitLocalCommand",
            FieldKey::PermitRemoteOpen => "PermitRemoteOpen",
            FieldKey::PKCS11Provider => "PKCS11Provider",
            FieldKey::Port => "Port",
            FieldKey::PreferredAuthentications => "PreferredAuthentications",
            FieldKey::ProxyCommand => "ProxyCommand",
            FieldKey::ProxyJump => "ProxyJump",
            FieldKey::ProxyUseFdpass => "ProxyUseFdpass",
            FieldKey::PubkeyAcceptedAlgorithms => "PubkeyAcceptedAlgorithms",
            FieldKey::PubkeyAuthentication => "PubkeyAuthentication",
            FieldKey::RekeyLimit => "RekeyLimit",
            FieldKey::RemoteCommand => "RemoteCommand",
            FieldKey::RemoteForward => "RemoteForward",
            FieldKey::RequestTTY => "RequestTTY",
            FieldKey::RequiredRSASize => "RequiredRSASize",
            FieldKey::RevokedHostKeys => "RevokedHostKeys",
            FieldKey::SecurityKeyProvider => "SecurityKeyProvider",
            FieldKey::SendEnv => "SendEnv",
            FieldKey::ServerAliveCountMax => "ServerAliveCountMax",
            FieldKey::ServerAliveInterval => "ServerAliveInterval",
            FieldKey::SessionType => "SessionType",
            FieldKey::SetEnv => "SetEnv",
            FieldKey::StdinNull => "StdinNull",
            FieldKey::StreamLocalBindMask => "StreamLocalBindMask",
            FieldKey::StreamLocalBindUnlink => "StreamLocalBindUnlink",
            FieldKey::StrictHostKeyChecking => "StrictHostKeyChecking",
            FieldKey::SyslogFacility => "SyslogFacility",
            FieldKey::TCPKeepAlive => "TCPKeepAlive",
            FieldKey::Tag => "Tag",
            FieldKey::Tunnel => "Tunnel",
            FieldKey::TunnelDevice => "TunnelDevice",
            FieldKey::UpdateHostKeys => "UpdateHostKeys",
            FieldKey::User => "User",
            FieldKey::UserKnownHostsFile => "UserKnownHostsFile",
            FieldKey::VerifyHostKeyDNS => "VerifyHostKeyDNS",
            FieldKey::VisualHostKey => "VisualHostKey",
            FieldKey::XAuthLocation => "XAuthLocation",
            FieldKey::Other(s) => s,
        }
    }
}

impl fmt::Display for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_canonical_str())
    }
}

impl FromStr for FieldKey {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FieldKey::parse(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!(FieldKey::parse("IdentityFile"), FieldKey::IdentityFile);
        assert_eq!(FieldKey::parse("identityfile"), FieldKey::IdentityFile);
        assert_eq!(FieldKey::parse("IDENTITYFILE"), FieldKey::IdentityFile);
    }

    #[test]
    fn unknown_becomes_other() {
        assert_eq!(
            FieldKey::parse("MadeUpOption"),
            FieldKey::Other("MadeUpOption".to_string())
        );
        assert_ne!(FieldKey::parse("a"), FieldKey::parse("A"));
    }

    #[test]
    fn cumulative_flags() {
        assert!(FieldKey::parse("IdentityFile").is_cumulative());
        assert!(FieldKey::parse("LocalForward").is_cumulative());
        assert!(!FieldKey::parse("User").is_cumulative());
        assert!(!FieldKey::parse("CanonicalDomains").is_cumulative());
    }

    #[test]
    fn display_roundtrips_canonical() {
        assert_eq!(FieldKey::Hostname.to_string(), "Hostname");
        assert_eq!(FieldKey::parse("hostname").to_string(), "Hostname");
    }
}
