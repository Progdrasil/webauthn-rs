//! # Webauthn-rs - Webauthn for Rust Server Applications
//!
//! Webauthn is a standard allowing communication between servers, browsers and authenticators
//! to allow strong, passwordless, cryptographic authentication to be performed. Webauthn
//! is able to operate with many authenticator types, such as U2F, TouchID, Windows Hello
//! and many more.
//!
//! This library aims to provide a secure Webauthn implementation that you can
//! plug into your application, so that you can provide Webauthn to your users.
//!
//! There are a number of focused use cases that this library provides, which are described in
//! the [WebauthnBuilder] and [Webauthn] struct.
//!
//! # Getting started
//!
//! In the simplest case where you just want to replace passwords with strong self contained multifactor
//! authentication, you should use our passkey flow.
//!
//! ```
//! use webauthn_rs::prelude::*;
//!
//! let rp_id = "example.com";
//! let rp_origin = Url::parse("https://idm.example.com")
//!     .expect("Invalid URL");
//! let mut builder = WebauthnBuilder::new(rp_id, &rp_origin)
//!     .expect("Invalid configuration");
//! let webauthn = builder.build()
//!     .expect("Invalid configuration");
//!
//! // Initiate a basic registration flow to enroll a cryptographic authenticator
//! let (ccr, skr) = webauthn
//!     .start_passkey_registration(
//!         Uuid::new_v4(),
//!         "claire",
//!         "Claire",
//!         None,
//!     )
//!     .expect("Failed to start registration.");
//! ```
//!
//! After this point you then need to use `finish_passkey_registration`, followed by
//! `start_passkey_authentication` and `finish_passkey_authentication`
//!
//! No other authentication factors are needed! A passkey combines inbuilt user verification (pin, biometrics, etc)
//! with a hardware cryptographic authenticator.
//!
//! # Tutorial
//!
//! Tutorials and examples on how to use this library in your website project is on the project github <https://github.com/kanidm/webauthn-rs/tree/master/tutorial>
//!
//! # What is a "Passkey"?
//!
//! Like all good things - "it depends". Mostly it depends who you ask, and at what time they adopted
//! the terminology. There are at least four definitions that we are aware of. A passkey is:
//!
//! * any possible webauthn authenticator - security key, tpm, touch id, etc
//! * a platform authenticator - built into a device such as touch id, tpm, etc
//! * a synchronised credential - backed by a cloud keychain like Apple iCloud
//! * a resident key - a stored, discoverable credential allowing usernameless flows
//!
//! The issue is each of these definitions have different pros/cons and different implications. For
//! example, passkeys as resident keys means you can accidentally brick many ctap2.0 devices by exhausting
//! their storage. Passkeys as platform authenticators means only certain devices can use them.
//! Passkeys as synced credentials means only certain devices with specific browser combinations can
//! use them.
//!
//! In this library we chose to define passkey's as "any possible authenticator". If the device
//! opportunistically creates rk (such as Apple iCloud Keychain) then in the future we *may* allow usernameless
//! flows once we are satisfied with the state of these ui's in browsers.
//!
//! # Features
//!
//! This library supports some optional features that you may wish to use. These are all
//! disabled by default as they have risks associated that you need to be aware of as an
//! authentication provider.
//!
//! ## Allow Serialising Registration and Authentication State
//!
//! During a webauthn registration or authentication ceremony, a random challenge is produced and
//! provided to the client. The full content of what is needed for the server to validate this
//! challenge is stored in the associated registration or authentication state types. This value
//! *MUST* be persisted on the server. If you store this in a cookie or some other form of client
//! side stored value, the client can replay a previous authentication state and signature without
//! possession of, or interaction with the authenticator, bypassing pretty much all of the security guarantees
//! of webauthn. Because of this risk by default these states are *not* allowed to be serialised
//! which prevents them from accidentally being placed into a cookie.
//!
//! However there are some *safe* cases of serialising these values. This includes serialising to
//! a database, or using a cookie "memory store" where the client side cookie is a key into a server-side
//! map or similar. Any of these prevent the replay attack threat.
//!
//! An alternate but "less good" method to mitigate replay attacks is to associate a very short
//! expiry window to the cookie if you need full client side state, but this may still allow some
//! forms of real time replay attacks to occur. We do not recommend this.
//!
//! Enabling the feature `danger-allow-state-serialisation` allows you to re-enable serialisation
//! of these types, provided you accept and understand the handling risks associated.
//!
//! ## Credential Internals and Type Changes
//!
//! By default the type wrappers around the keys are opaque. However in some cases you
//! may wish to migrate a key between types (security key to passkey, attested_passkey to passkey)
//! for example. Alternately, you may wish to access the internals of a credential to implement
//! an alternate serialisation or storage mechanism. In these cases you can access the underlying
//! [Credential] type via Into and From by enabling the feature `danger-credential-internals`. The
//! [Credential] type is exposed via the [prelude] when this feature is enabled.
//!
//! However, you should be aware that manipulating the internals of a [Credential] may affect the usage
//! of that [Credential] in certain use cases. You should be careful when enabling this feature that
//! you do not change internal [Credential] values.
//!
//! ## User-Presence only SecurityKeys
//!
//! By default, SecurityKeys will opportunistically enforce User Verification (Such as a PIN or
//! Biometric). This can cause issues with Firefox which only supports CTAP1. An example of this
//! is if you register a SecurityKey on chromium it will be bound to always perform UserVerification
//! for the life of the SecurityKey precluding it's use on Firefox.
//!
//! Enabling the feature `danger-user-presence-only-security-keys` changes these keys to prevent
//! User Verification if possible. However, newer keys will confusingly force a User Verification
//! on registration, but will then not prompt for this during usage. Some user surveys have shown
//! this to confuse users to why the UV is not requested, and it can lower trust in these tokens
//! when they are elevated to be self-contained MFA as the user believes these UV prompts to be
//! unreliable and not verified correctly - in other words it trains users to believe that these
//! prompts do nothing and have no effect. In these cases you MUST communicate to the user that
//! the UV *may* occur on registration and then will not occur again, and that is *by design*.
//!
//! If in doubt, do not enable this feature.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(warnings)]
#![warn(unused_extern_crates)]
#![warn(missing_docs)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unreachable)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::trivially_copy_pass_by_ref)]

#[macro_use]
extern crate tracing;

mod interface;

use url::Url;
use uuid::Uuid;
use webauthn_rs_core::error::{WebauthnError, WebauthnResult};
use webauthn_rs_core::proto::*;
use webauthn_rs_core::WebauthnCore;

use crate::interface::*;

/// A prelude of types that are used by `Webauthn`
pub mod prelude {
    pub use crate::interface::*;
    pub use crate::{Webauthn, WebauthnBuilder};
    pub use base64urlsafedata::Base64UrlSafeData;
    pub use url::Url;
    pub use uuid::Uuid;
    pub use webauthn_rs_core::error::{WebauthnError, WebauthnResult};
    #[cfg(feature = "danger-credential-internals")]
    pub use webauthn_rs_core::proto::Credential;
    pub use webauthn_rs_core::proto::{AttestationCa, AttestationCaList, AuthenticatorAttachment};
    pub use webauthn_rs_core::proto::{
        AttestationMetadata, AuthenticationResult, AuthenticationState, CreationChallengeResponse,
        CredentialID, ParsedAttestation, ParsedAttestationData, PublicKeyCredential,
        RegisterPublicKeyCredential, RequestChallengeResponse,
    };
    pub use webauthn_rs_core::proto::{
        COSEAlgorithm, COSEEC2Key, COSEKey, COSEKeyType, COSEKeyTypeId, COSEOKPKey, COSERSAKey,
        ECDSACurve, EDDSACurve,
    };
    pub use webauthn_rs_core::AttestationFormat;
}

/// A constructor for a new [Webauthn] instance. This accepts and configures a number of site-wide
/// properties that apply to all webauthn operations of this service.
#[derive(Debug)]
pub struct WebauthnBuilder<'a> {
    rp_name: Option<&'a str>,
    rp_id: &'a str,
    allowed_origins: Vec<Url>,
    allow_subdomains: bool,
    allow_any_port: bool,
    algorithms: Vec<COSEAlgorithm>,
    user_presence_only_security_keys: bool,
}

impl<'a> WebauthnBuilder<'a> {
    /// Initiate a new builder. This takes the relying party id and relying party origin.
    ///
    /// # Safety
    ///
    /// rp_id is what Credentials (Authenticators) bind themself to - rp_id can NOT be changed
    /// without breaking all of your users' associated credentials in the future!
    ///
    /// # Examples
    ///
    /// ```
    /// use webauthn_rs::prelude::*;
    ///
    /// let rp_id = "example.com";
    /// let rp_origin = Url::parse("https://idm.example.com")
    ///     .expect("Invalid URL");
    /// let mut builder = WebauthnBuilder::new(rp_id, &rp_origin)
    ///     .expect("Invalid configuration");
    /// ```
    ///
    /// # Errors
    ///
    /// rp_id *must* be an effective domain of rp_origin. This means that if you are hosting
    /// `https://idm.example.com`, rp_id must be `idm.example.com`, `example.com` or `com`.
    ///
    /// ```
    /// use webauthn_rs::prelude::*;
    ///
    /// let rp_id = "example.com";
    /// let rp_origin = Url::parse("https://idm.different.com")
    ///     .expect("Invalid URL");
    /// assert!(WebauthnBuilder::new(rp_id, &rp_origin).is_err());
    /// ```
    pub fn new(rp_id: &'a str, rp_origin: &'a Url) -> WebauthnResult<Self> {
        // Check the rp_name and rp_id.
        let valid = rp_origin
            .domain()
            .map(|effective_domain| {
                // We need to prepend the '.' here to ensure that myexample.com != example.com,
                // rather than just ends with.
                effective_domain.ends_with(&format!(".{rp_id}")) || effective_domain == rp_id
            })
            .unwrap_or(false);

        if valid {
            Ok(WebauthnBuilder {
                rp_name: None,
                rp_id,
                allowed_origins: vec![rp_origin.to_owned()],
                allow_subdomains: false,
                allow_any_port: false,
                algorithms: COSEAlgorithm::secure_algs(),
                user_presence_only_security_keys: false,
            })
        } else {
            error!("rp_id is not an effective_domain of rp_origin");
            Err(WebauthnError::Configuration)
        }
    }

    /// Setting this flag to true allows subdomains to be considered valid in Webauthn operations.
    /// An example of this is if you wish for `https://au.idm.example.com` to be a valid domain
    /// for Webauthn when the configuration is `https://idm.example.com`. Generally this occurs
    /// when you have a centralised IDM system, but location specific systems with DNS based
    /// redirection or routing.
    ///
    /// If in doubt, do NOT change this value. Defaults to "false".
    pub fn allow_subdomains(mut self, allow: bool) -> Self {
        self.allow_subdomains = allow;
        self
    }

    /// Setting this flag skips port checks on origin matches
    pub fn allow_any_port(mut self, allow: bool) -> Self {
        self.allow_any_port = allow;
        self
    }

    /// Set extra origins to be considered valid in Webauthn operations. A common example of this is
    /// enabling use with iOS or Android native "webauthn-like" APIs, which return different
    /// app-specific origins than a web browser would.
    pub fn append_allowed_origin(mut self, origin: &Url) -> Self {
        self.allowed_origins.push(origin.to_owned());
        self
    }

    /// Set the relying party name. This may be shown to the user. This value can be changed in
    /// the future without affecting credentials that have already registered.
    ///
    /// If not set, defaults to rp_id.
    pub fn rp_name(mut self, rp_name: &'a str) -> Self {
        self.rp_name = Some(rp_name);
        self
    }

    /// Enable security keys to only require user presence, rather than enforcing
    /// their user-verification state.
    ///
    /// *requires feature danger-user-presence-only-security-keys*
    #[cfg(feature = "danger-user-presence-only-security-keys")]
    pub fn danger_set_user_presence_only_security_keys(mut self, enable: bool) -> Self {
        self.user_presence_only_security_keys = enable;
        self
    }

    /// Complete the construction of the [Webauthn] instance. If an invalid configuration setting
    /// is found, an Error will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use webauthn_rs::prelude::*;
    ///
    /// let rp_id = "example.com";
    /// let rp_origin = Url::parse("https://idm.example.com")
    ///     .expect("Invalid URL");
    /// let mut builder = WebauthnBuilder::new(rp_id, &rp_origin)
    ///     .expect("Invalid configuration");
    /// let webauthn = builder.build()
    ///     .expect("Invalid configuration");
    /// ```
    pub fn build(self) -> WebauthnResult<Webauthn> {
        Ok(Webauthn {
            core: WebauthnCore::new_unsafe_experts_only(
                self.rp_name.unwrap_or(self.rp_id),
                self.rp_id,
                self.allowed_origins,
                None,
                Some(self.allow_subdomains),
                Some(self.allow_any_port),
            ),
            algorithms: self.algorithms,
            user_presence_only_security_keys: self.user_presence_only_security_keys,
        })
    }
}

/// An instance of a Webauthn site. This is the main point of interaction for registering and
/// authenticating credentials for users. Depending on your needs, you'll want to allow users
/// to register and authenticate with different kinds of authenticators.
///
/// __I just want to replace passwords with strong cryptographic authentication, and I don't have other requirements__
///
/// > You should use [`start_passkey_registration`](Webauthn::start_passkey_registration)
///
///
/// __I want to replace passwords with strong multi-factor cryptographic authentication, limited to
/// a known set of controlled and trusted authenticator types__
///
/// This type requires `preview-features` enabled as the current form of the Attestation CA List
/// may change in the future.
///
/// > You should use [`start_attested_passkey_registration`](Webauthn::start_attested_passkey_registration)
///
///
/// __I want users to have their identites stored on their devices, and for them to authenticate with
/// strong multi-factor cryptographic authentication limited to a known set of trusted authenticator types__
///
/// This authenticator type consumes limited storage space on users' authenticators, and may result in failures or device
/// bricking.
/// You **MUST** only use it in tightly controlled environments where you supply devices to your
/// users.
///
/// > You should use [`start_attested_resident_key_registration`](Webauthn::start_attested_resident_key_registration) (still in development)
///
///
/// __I want a security token along with an external password to create multi-factor authentication__
///
/// If possible, consider [`start_passkey_registration`](Webauthn::start_passkey_registration) OR
/// [`start_attested_passkey_registration`](Webauthn::start_attested_passkey_registration)
/// instead - it's likely to provide a better user experience over security keys as MFA!
///
/// > If you really want a security key, you should use [`start_securitykey_registration`](Webauthn::start_securitykey_registration)
///
#[derive(Debug, Clone)]
pub struct Webauthn {
    core: WebauthnCore,
    algorithms: Vec<COSEAlgorithm>,
    user_presence_only_security_keys: bool,
}

impl Webauthn {
    /// Get the currently configured origins
    pub fn get_allowed_origins(&self) -> &[Url] {
        self.core.get_allowed_origins()
    }

    /// Initiate the registration of a new passkey for a user. A passkey is any cryptographic
    /// authenticator acting as a single factor of authentication, far stronger than a password
    /// or email-reset link.
    ///
    /// Some examples of passkeys include Yubikeys, TouchID, FaceID, Windows Hello and others.
    ///
    /// The keys *may* exist and 'roam' between multiple devices. For example, Apple allows Passkeys
    /// to sync between devices owned by the same Apple account. This can affect your risk model
    /// related to these credentials, but generally in all cases passkeys are better than passwords!
    ///
    /// You *should* NOT pair this authentication with another factor. A passkey may opportunistically
    /// allow and enforce user-verification (MFA), but this is NOT guaranteed with all authenticator
    /// types.
    ///
    /// `user_unique_id` *may* be stored in the authenticator. This may allow the credential to
    ///  identify the user during certain client side work flows.
    ///
    /// `user_name` and `user_display_name` *may* be stored in the authenticator. `user_name` is a
    /// friendly account name such as "claire@example.com". `user_display_name` is the persons chosen
    /// way to be identified such as "Claire". Both can change at *any* time on the client side, and
    /// MUST NOT be used as primary keys. They *may not* be present in authentication, these are only
    /// present to allow client facing work flows to display human friendly identifiers.
    ///
    /// `exclude_credentials` ensures that a set of credentials may not participate in this registration.
    /// You *should* provide the list of credentials that are already registered to this user's account
    /// to prevent duplicate credential registrations. These credentials *can* be from different
    /// authenticator classes since we only require the `CredentialID`
    ///
    /// # Returns
    ///
    /// This function returns a `CreationChallengeResponse` which you must serialise to json and
    /// send to the user agent (e.g. a browser) for it to conduct the registration. You must persist
    /// on the server the `PasskeyRegistration` which contains the state of this registration
    /// attempt and is paired to the `CreationChallengeResponse`.
    ///
    /// Finally you need to call [`finish_passkey_registration`](Webauthn::finish_passkey_registration)
    /// to complete the registration.
    ///
    /// WARNING ⚠️  YOU MUST STORE THE [PasskeyRegistration] VALUE SERVER SIDE.
    ///
    /// Failure to do so *may* open you to replay attacks which can significantly weaken the
    /// security of this system.
    ///
    /// ```
    /// # use webauthn_rs::prelude::*;
    ///
    /// # let rp_id = "example.com";
    /// # let rp_origin = Url::parse("https://idm.example.com")
    /// #     .expect("Invalid URL");
    /// # let mut builder = WebauthnBuilder::new(rp_id, &rp_origin)
    /// #     .expect("Invalid configuration");
    /// # let webauthn = builder.build()
    /// #     .expect("Invalid configuration");
    ///
    /// // you must store this user's unique id with the account. Alternatelly you can
    /// // use an existed UUID source.
    /// let user_unique_id = Uuid::new_v4();
    ///
    /// // Initiate a basic registration flow, allowing any cryptograhpic authenticator to proceed.
    /// let (ccr, skr) = webauthn
    ///     .start_passkey_registration(
    ///         user_unique_id,
    ///         "claire",
    ///         "Claire",
    ///         None, // No other credentials are registered yet.
    ///     )
    ///     .expect("Failed to start registration.");
    /// ```
    pub fn start_passkey_registration(
        &self,
        user_unique_id: Uuid,
        user_name: &str,
        user_display_name: &str,
        exclude_credentials: Option<Vec<CredentialID>>,
    ) -> WebauthnResult<(CreationChallengeResponse, PasskeyRegistration)> {
        let attestation = AttestationConveyancePreference::None;
        let credential_algorithms = self.algorithms.clone();
        let require_resident_key = false;
        let authenticator_attachment = None;
        let policy = Some(UserVerificationPolicy::Required);
        let reject_passkeys = false;

        let extensions = Some(RequestRegistrationExtensions {
            cred_protect: Some(CredProtect {
                // Since this may contain PII, we want to enforce this. We also
                // want the device to strictly enforce it's UV state.
                credential_protection_policy: CredentialProtectionPolicy::UserVerificationRequired,
                // If set to true, causes many authenticators to shit the bed. We have to just hope
                // and pray instead. This is because many device classes when they see this extension
                // and can't satisfy it, they fail the operation instead.
                enforce_credential_protection_policy: Some(false),
            }),
            uvm: Some(true),
            cred_props: Some(true),
            min_pin_length: None,
            hmac_create_secret: None,
        });

        self.core
            .generate_challenge_register_options(
                user_unique_id.as_bytes(),
                user_name,
                user_display_name,
                attestation,
                policy,
                exclude_credentials,
                extensions,
                credential_algorithms,
                require_resident_key,
                authenticator_attachment,
                reject_passkeys,
            )
            .map(|(ccr, rs)| (ccr, PasskeyRegistration { rs }))
    }

    /// Complete the registration of the credential. The user agent (e.g. a browser) will return the data of `RegisterPublicKeyCredential`,
    /// and the server provides its paired [PasskeyRegistration]. The details of the Authenticator
    /// based on the registration parameters are asserted.
    ///
    /// # Errors
    /// If any part of the registration is incorrect or invalid, an error will be returned. See [WebauthnError].
    ///
    /// # Returns
    ///
    /// The returned `Passkey` must be associated to the users account, and is used for future
    /// authentications via [`start_passkey_authentication`](Webauthn::start_passkey_authentication).
    ///
    /// You MUST assert that the registered `CredentialID` has not previously been registered.
    /// to any other account.
    pub fn finish_passkey_registration(
        &self,
        reg: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> WebauthnResult<Passkey> {
        self.core
            .register_credential(reg, &state.rs, None)
            .map(|cred| Passkey { cred })
    }

    /// Given a set of `Passkey`'s, begin an authentication of the user. This returns
    /// a `RequestChallengeResponse`, which should be serialised to json and sent to the user agent (e.g. a browser).
    /// The server must persist the [PasskeyAuthentication] state as it is paired to the
    /// `RequestChallengeResponse` and required to complete the authentication.
    ///
    /// Finally you need to call [`finish_passkey_authentication`](Webauthn::finish_passkey_authentication)
    /// to complete the authentication.
    ///
    /// WARNING ⚠️  YOU MUST STORE THE [PasskeyAuthentication] VALUE SERVER SIDE.
    ///
    /// Failure to do so *may* open you to replay attacks which can significantly weaken the
    /// security of this system.
    pub fn start_passkey_authentication(
        &self,
        creds: &[Passkey],
    ) -> WebauthnResult<(RequestChallengeResponse, PasskeyAuthentication)> {
        let extensions = None;
        let creds = creds.iter().map(|sk| sk.cred.clone()).collect();
        let policy = UserVerificationPolicy::Required;
        let allow_backup_eligible_upgrade = true;

        self.core
            .generate_challenge_authenticate_policy(
                creds,
                policy,
                extensions,
                allow_backup_eligible_upgrade,
            )
            .map(|(rcr, ast)| (rcr, PasskeyAuthentication { ast }))
    }

    /// Given the `PublicKeyCredential` returned by the user agent (e.g. a browser), and the stored [PasskeyAuthentication]
    /// complete the authentication of the user.
    ///
    /// # Errors
    /// If any part of the registration is incorrect or invalid, an error will be returned. See [WebauthnError].
    ///
    /// # Returns
    /// On success, [AuthenticationResult] is returned which contains some details of the Authentication
    /// process.
    ///
    /// As per <https://www.w3.org/TR/webauthn-3/#sctn-verifying-assertion> 21:
    ///
    /// If the Credential Counter is greater than 0 you MUST assert that the counter is greater than
    /// the stored counter. If the counter is equal or less than this MAY indicate a cloned credential
    /// and you SHOULD invalidate and reject that credential as a result.
    ///
    /// From this [AuthenticationResult] you *should* update the Credential's Counter value if it is
    /// valid per the above check. If you wish
    /// you *may* use the content of the [AuthenticationResult] for extended validations (such as the
    /// presence of the user verification flag).
    pub fn finish_passkey_authentication(
        &self,
        reg: &PublicKeyCredential,
        state: &PasskeyAuthentication,
    ) -> WebauthnResult<AuthenticationResult> {
        self.core.authenticate_credential(reg, &state.ast)
    }

    /// Initiate the registration of a new security key for a user. A security key is any cryptographic
    /// authenticator acting as a single factor of authentication to supplement a password or some
    /// other authentication factor.
    ///
    /// Some examples of security keys include Yubikeys, Feitian ePass, and others.
    ///
    /// We don't recommend this over [Passkey] or [AttestedPasskey], as today in Webauthn most devices
    /// due to their construction require userVerification to be maintained for user trust. What this
    /// means is that most users will require a password, their security key, and a pin or biometric
    /// on the security key for a total of three factors. This adds friction to the user experience
    /// but is required due to a consistency flaw in CTAP2.0 and newer devices. Since the user already
    /// needs a pin or biometrics, why not just use the device as a self contained MFA?
    ///
    /// You MUST pair this authentication with another factor. A security key may opportunistically
    /// allow and enforce user-verification (MFA), but this is NOT guaranteed.
    ///
    /// `user_unique_id` *may* be stored in the authenticator. This may allow the credential to
    ///  identify the user during certain client side work flows.
    ///
    /// `user_name` and `user_display_name` *may* be stored in the authenticator. `user_name` is a
    /// friendly account name such as "claire@example.com". `user_display_name` is the persons chosen
    /// way to be identified such as "Claire". Both can change at *any* time on the client side, and
    /// MUST NOT be used as primary keys. They *may not* be present in authentication, these are only
    /// present to allow client work flows to display human friendly identifiers.
    ///
    /// `exclude_credentials` ensures that a set of credentials may not participate in this registration.
    /// You *should* provide the list of credentials that are already registered to this user's account
    /// to prevent duplicate credential registrations.
    ///
    /// `attestation_ca_list` contains an optional list of Root CA certificates of authenticator
    /// manufacturers that you wish to trust. For example, if you want to only allow Yubikeys on
    /// your site, then you can provide the Yubico Root CA in this list, to validate that all
    /// registered devices are manufactured by Yubico.
    ///
    /// Extensions may ONLY be accessed if an `attestation_ca_list` is provided, else they can
    /// ARE NOT trusted.
    ///
    /// # Returns
    ///
    /// This function returns a `CreationChallengeResponse` which you must serialise to json and
    /// send to the user agent (e.g. a browser) for it to conduct the registration. You must persist
    /// on the server the [SecurityKeyRegistration] which contains the state of this registration
    /// attempt and is paired to the `CreationChallengeResponse`.
    ///
    /// Finally you need to call [`finish_securitykey_registration`](Webauthn::finish_securitykey_registration)
    /// to complete the registration.
    ///
    /// WARNING ⚠️  YOU MUST STORE THE [SecurityKeyRegistration] VALUE SERVER SIDE.
    ///
    /// Failure to do so *may* open you to replay attacks which can significantly weaken the
    /// security of this system.
    ///
    /// ```
    /// # use webauthn_rs::prelude::*;
    ///
    /// # let rp_id = "example.com";
    /// # let rp_origin = Url::parse("https://idm.example.com")
    /// #     .expect("Invalid URL");
    /// # let mut builder = WebauthnBuilder::new(rp_id, &rp_origin)
    /// #     .expect("Invalid configuration");
    /// # let webauthn = builder.build()
    /// #     .expect("Invalid configuration");
    ///
    /// // you must store this user's unique id with the account. Alternatelly you can
    /// // use an existed UUID source.
    /// let user_unique_id = Uuid::new_v4();
    ///
    /// // Initiate a basic registration flow, allowing any cryptograhpic authenticator to proceed.
    /// let (ccr, skr) = webauthn
    ///     .start_securitykey_registration(
    ///         user_unique_id,
    ///         "claire",
    ///         "Claire",
    ///         None,
    ///         None,
    ///         None,
    ///     )
    ///     .expect("Failed to start registration.");
    ///
    /// // Initiate a basic registration flow, hinting that the device is probably roaming (i.e. a usb),
    /// // but it could have any attachement in reality
    /// let (ccr, skr) = webauthn
    ///     .start_securitykey_registration(
    ///         Uuid::new_v4(),
    ///         "claire",
    ///         "Claire",
    ///         None,
    ///         None,
    ///         Some(AuthenticatorAttachment::CrossPlatform),
    ///     )
    ///     .expect("Failed to start registration.");
    ///
    /// // Only allow credentials from manufacturers that are trusted and part of the webauthn-rs
    /// // strict "high quality" list.
    ///
    /// use webauthn_rs_device_catalog::Data;
    /// let device_catalog = Data::strict();
    ///
    /// let attestation_ca_list = (&device_catalog)
    ///     .try_into()
    ///     .expect("Failed to build attestation ca list");
    ///
    /// let (ccr, skr) = webauthn
    ///     .start_securitykey_registration(
    ///         Uuid::new_v4(),
    ///         "claire",
    ///         "Claire",
    ///         None,
    ///         Some(attestation_ca_list),
    ///         None,
    ///     )
    ///     .expect("Failed to start registration.");
    /// ```
    pub fn start_securitykey_registration(
        &self,
        user_unique_id: Uuid,
        user_name: &str,
        user_display_name: &str,
        exclude_credentials: Option<Vec<CredentialID>>,
        attestation_ca_list: Option<AttestationCaList>,
        ui_hint_authenticator_attachment: Option<AuthenticatorAttachment>,
    ) -> WebauthnResult<(CreationChallengeResponse, SecurityKeyRegistration)> {
        let attestation = if let Some(ca_list) = attestation_ca_list.as_ref() {
            if ca_list.is_empty() {
                return Err(WebauthnError::MissingAttestationCaList);
            } else {
                AttestationConveyancePreference::Direct
            }
        } else {
            AttestationConveyancePreference::None
        };

        let cred_protect = if self.user_presence_only_security_keys {
            None
        } else {
            Some(CredProtect {
                // We want the device to strictly enforce it's UV state.
                credential_protection_policy: CredentialProtectionPolicy::UserVerificationRequired,
                // If set to true, causes many authenticators to shit the bed. Since this type doesn't
                // have the same strict rules about attestation, then we just use this opportunistically.
                enforce_credential_protection_policy: Some(false),
            })
        };

        let extensions = Some(RequestRegistrationExtensions {
            cred_protect,
            uvm: Some(true),
            cred_props: Some(true),
            min_pin_length: None,
            hmac_create_secret: None,
        });

        let credential_algorithms = self.algorithms.clone();
        let require_resident_key = false;
        let policy = if self.user_presence_only_security_keys {
            Some(UserVerificationPolicy::Discouraged_DO_NOT_USE)
        } else {
            Some(UserVerificationPolicy::Preferred)
        };
        let reject_passkeys = true;

        self.core
            .generate_challenge_register_options(
                user_unique_id.as_bytes(),
                user_name,
                user_display_name,
                attestation,
                policy,
                exclude_credentials,
                extensions,
                credential_algorithms,
                require_resident_key,
                ui_hint_authenticator_attachment,
                reject_passkeys,
            )
            .map(|(ccr, rs)| {
                (
                    ccr,
                    SecurityKeyRegistration {
                        rs,
                        ca_list: attestation_ca_list,
                    },
                )
            })
    }

    /// Complete the registration of the credential. The user agent (e.g. a browser) will return the data of `RegisterPublicKeyCredential`,
    /// and the server provides it's paired [SecurityKeyRegistration]. The details of the Authenticator
    /// based on the registration parameters are asserted.
    ///
    /// # Errors
    /// If any part of the registration is incorrect or invalid, an error will be returned. See [WebauthnError].
    ///
    /// # Returns
    ///
    /// The returned [SecurityKey] must be associated to the users account, and is used for future
    /// authentications via (`start_securitykey_authentication`)[crate::Webauthn::start_securitykey_authentication].
    ///
    /// You MUST assert that the registered [CredentialID] has not previously been registered.
    /// to any other account.
    ///
    /// # Verifying specific device models
    /// If you wish to assert a specifc type of device model is in use, you can inspect the
    /// SecurityKey `attestation()` and it's associated metadata. You can use this to check for
    /// specific device aaguids for example.
    ///
    pub fn finish_securitykey_registration(
        &self,
        reg: &RegisterPublicKeyCredential,
        state: &SecurityKeyRegistration,
    ) -> WebauthnResult<SecurityKey> {
        self.core
            .register_credential(reg, &state.rs, state.ca_list.as_ref())
            .map(|cred| SecurityKey { cred })
    }

    /// Given a set of [SecurityKey], begin an authentication of the user. This returns
    /// a `RequestChallengeResponse`, which should be serialised to json and sent to the user agent (e.g. a browser).
    /// The server must persist the [SecurityKeyAuthentication] state as it is paired to the
    /// `RequestChallengeResponse` and required to complete the authentication.
    ///
    /// Finally you need to call [`finish_securitykey_authentication`](Webauthn::finish_securitykey_authentication)
    /// to complete the authentication.
    ///
    /// WARNING ⚠️  YOU MUST STORE THE [SecurityKeyAuthentication] VALUE SERVER SIDE.
    ///
    /// Failure to do so *may* open you to replay attacks which can significantly weaken the
    /// security of this system.
    pub fn start_securitykey_authentication(
        &self,
        creds: &[SecurityKey],
    ) -> WebauthnResult<(RequestChallengeResponse, SecurityKeyAuthentication)> {
        let extensions = None;
        let creds = creds.iter().map(|sk| sk.cred.clone()).collect();
        let allow_backup_eligible_upgrade = false;

        let policy = if self.user_presence_only_security_keys {
            UserVerificationPolicy::Discouraged_DO_NOT_USE
        } else {
            UserVerificationPolicy::Preferred
        };

        self.core
            .generate_challenge_authenticate_policy(
                creds,
                policy,
                extensions,
                allow_backup_eligible_upgrade,
            )
            .map(|(rcr, ast)| (rcr, SecurityKeyAuthentication { ast }))
    }

    /// Given the `PublicKeyCredential` returned by the user agent (e.g. a browser), and the stored [SecurityKeyAuthentication]
    /// complete the authentication of the user.
    ///
    /// # Errors
    /// If any part of the registration is incorrect or invalid, an error will be returned. See [WebauthnError].
    ///
    /// # Returns
    /// On success, [AuthenticationResult] is returned which contains some details of the Authentication
    /// process.
    ///
    /// You should use `SecurityKey::update_credential` on the returned [AuthenticationResult] and
    /// ensure it is persisted.
    pub fn finish_securitykey_authentication(
        &self,
        reg: &PublicKeyCredential,
        state: &SecurityKeyAuthentication,
    ) -> WebauthnResult<AuthenticationResult> {
        self.core.authenticate_credential(reg, &state.ast)
    }
}

#[cfg(feature = "preview-features")]
impl Webauthn {
    /// Initiate the registration of a new attested_passkey key for a user. A attested_passkey key is a
    /// cryptographic authenticator that is a self-contained multifactor authenticator. This means
    /// that the device (such as a yubikey) verifies the user is who they say they are via a PIN,
    /// biometric or other factor. Only if this verification passes, is the signature released
    /// and provided.
    ///
    /// As a result, the server *only* requires this attested_passkey key to authenticator the user
    /// and assert their identity. Because of this reliance on the authenticator, attestation of
    /// the authenticator and it's properties is strongly recommended.
    ///
    /// The primary difference to a passkey, is that these credentials *can not* 'roam' between multiple
    /// devices, and must be bound to a single authenticator. This precludes the use of certain types
    /// of authenticators (such as Apple's Passkeys as these are always synced).
    ///
    /// Additionally, these credentials can have an attestation or certificate of authenticity
    /// validated to give you stronger assertions in the types of devices in use.
    ///
    /// You *should* recommend to the user to register multiple attested_passkey keys to their account on
    /// seperate devices so that they have fall back authentication.
    ///
    /// You *should* have a workflow that allows a user to register new devices without a need to register
    /// other factors. For example, allow a QR code that can be scanned from a phone, or a one-time
    /// link that can be copied to the device.
    ///
    /// You *must* have a recovery workflow in case all devices are lost or destroyed.
    ///
    /// `user_unique_id` *may* be stored in the authenticator. This may allow the credential to
    ///  identify the user during certain client side work flows.
    ///
    /// `user_name` and `user_display_name` *may* be stored in the authenticator. `user_name` is a
    /// friendly account name such as "claire@example.com". `user_display_name` is the persons chosen
    /// way to be identified such as "Claire". Both can change at *any* time on the client side, and
    /// MUST NOT be used as primary keys. They *may not* be present in authentication, these are only
    /// present to allow client work flows to display human friendly identifiers.
    ///
    /// `exclude_credentials` ensures that a set of credentials may not participate in this registration.
    /// You *should* provide the list of credentials that are already registered to this user's account
    /// to prevent duplicate credential registrations.
    ///
    /// `attestation_ca_list` contains an optional list of Root CA certificates of authenticator
    /// manufacturers that you wish to trust. For example, if you want to only allow Yubikeys on
    /// your site, then you can provide the Yubico Root CA in this list, to validate that all
    /// registered devices are manufactured by Yubico.
    ///
    /// `ui_hint_authenticator_attachment` provides a UX/UI hint to the browser about the types
    /// of credentials that could be used in this registration. If set to `None` all authenticator
    /// attachement classes are valid. If set to Platform, only authenticators that are part of the
    /// device are used such as a TPM or TouchId. If set to Cross-Platform, only devices that are
    /// removable from the device can be used such as yubikeys.
    ///
    /// Currently, extensions are *not* possible to request due to webauthn not properly supporting
    /// them in broader contexts.
    ///
    /// # Returns
    ///
    /// This function returns a `CreationChallengeResponse` which you must serialise to json and
    /// send to the user agent (e.g. a browser) for it to conduct the registration. You must persist
    /// on the server the `AttestedPasskeyRegistration` which contains the state of this registration
    /// attempt and is paired to the `CreationChallengeResponse`.
    ///
    /// Finally you need to call [`finish_attested_passkey_registration`](Webauthn::finish_attested_passkey_registration)
    /// to complete the registration.
    ///
    /// WARNING ⚠️  YOU MUST STORE THE [AttestedPasskeyRegistration] VALUE SERVER SIDE.
    ///
    /// Failure to do so *may* open you to replay attacks which can significantly weaken the
    /// security of this system.
    ///
    /// ```
    /// # use webauthn_rs::prelude::*;
    /// use webauthn_rs_device_catalog::Data;
    ///
    /// # let rp_id = "example.com";
    /// # let rp_origin = Url::parse("https://idm.example.com")
    /// #     .expect("Invalid url");
    /// # let mut builder = WebauthnBuilder::new(rp_id, &rp_origin)
    /// #     .expect("Invalid configuration");
    /// # let webauthn = builder.build()
    /// #     .expect("Invalid configuration");
    ///
    /// // you must store this user's unique id with the account. Alternatelly you can
    /// // use an existed UUID source.
    /// let user_unique_id = Uuid::new_v4();
    ///
    /// // Create a device catalog reference that contains a list of known high quality authenticators
    /// let device_catalog = Data::all_known_devices();
    ///
    /// let attestation_ca_list = (&device_catalog)
    ///     .try_into()
    ///     .expect("Failed to build attestation ca list");
    ///
    /// // Initiate a basic registration flow, allowing any attested cryptograhpic authenticator to proceed.
    /// // Hint (but do not enforce) that we prefer this to be a token/key like a yubikey.
    /// // To enforce this you can validate the properties of the returned device aaguid.
    /// let (ccr, skr) = webauthn
    ///     .start_attested_passkey_registration(
    ///         user_unique_id,
    ///         "claire",
    ///         "Claire",
    ///         None,
    ///         attestation_ca_list,
    ///         Some(AuthenticatorAttachment::CrossPlatform),
    ///     )
    ///     .expect("Failed to start registration.");
    ///
    /// // Only allow credentials from manufacturers that are trusted and part of the webauthn-rs
    /// // strict "high quality" list.
    /// // Hint (but do not enforce) that we prefer this to be a device like TouchID.
    /// // To enforce this you can validate the attestation ca used along with the returned device aaguid
    ///
    /// let device_catalog = Data::strict();
    ///
    /// let attestation_ca_list = (&device_catalog)
    ///     .try_into()
    ///     .expect("Failed to build attestation ca list");
    ///
    /// let (ccr, skr) = webauthn
    ///     .start_attested_passkey_registration(
    ///         Uuid::new_v4(),
    ///         "claire",
    ///         "Claire",
    ///         None,
    ///         attestation_ca_list,
    ///         Some(AuthenticatorAttachment::Platform),
    ///     )
    ///     .expect("Failed to start registration.");
    /// ```
    pub fn start_attested_passkey_registration(
        &self,
        user_unique_id: Uuid,
        user_name: &str,
        user_display_name: &str,
        exclude_credentials: Option<Vec<CredentialID>>,
        attestation_ca_list: AttestationCaList,
        ui_hint_authenticator_attachment: Option<AuthenticatorAttachment>,
        // extensions
    ) -> WebauthnResult<(CreationChallengeResponse, AttestedPasskeyRegistration)> {
        let attestation = AttestationConveyancePreference::Direct;
        if attestation_ca_list.is_empty() {
            return Err(WebauthnError::MissingAttestationCaList);
        }

        let credential_algorithms = self.algorithms.clone();
        let require_resident_key = false;
        let policy = Some(UserVerificationPolicy::Required);
        let reject_passkeys = true;

        let extensions = Some(RequestRegistrationExtensions {
            cred_protect: Some(CredProtect {
                // Since this may contain PII, we need to enforce this. We also
                // want the device to strictly enforce it's UV state.
                credential_protection_policy: CredentialProtectionPolicy::UserVerificationRequired,
                // Set to true since this function requires attestation, and attestation is really
                // only viable on FIDO2/CTAP2 creds that actually support this.
                enforce_credential_protection_policy: Some(true),
            }),
            uvm: Some(true),
            cred_props: Some(true),
            min_pin_length: Some(true),
            hmac_create_secret: None,
        });

        self.core
            .generate_challenge_register_options(
                user_unique_id.as_bytes(),
                user_name,
                user_display_name,
                attestation,
                policy,
                exclude_credentials,
                extensions,
                credential_algorithms,
                require_resident_key,
                ui_hint_authenticator_attachment,
                reject_passkeys,
            )
            .map(|(ccr, rs)| {
                (
                    ccr,
                    AttestedPasskeyRegistration {
                        rs,
                        ca_list: attestation_ca_list,
                    },
                )
            })
    }

    /// Complete the registration of the credential. The user agent (e.g. a browser) will return the data of `RegisterPublicKeyCredential`,
    /// and the server provides it's paired [AttestedPasskeyRegistration]. The details of the Authenticator
    /// based on the registration parameters are asserted.
    ///
    /// # Errors
    /// If any part of the registration is incorrect or invalid, an error will be returned. See [WebauthnError].
    ///
    /// # Returns
    /// The returned [AttestedPasskey] must be associated to the users account, and is used for future
    /// authentications via [crate::Webauthn::start_attested_passkey_authentication].
    ///
    /// # Verifying specific device models
    /// If you wish to assert a specifc type of device model is in use, you can inspect the
    /// AttestedPasskey `attestation()` and it's associated metadata. You can use this to check for
    /// specific device aaguids for example.
    ///
    pub fn finish_attested_passkey_registration(
        &self,
        reg: &RegisterPublicKeyCredential,
        state: &AttestedPasskeyRegistration,
    ) -> WebauthnResult<AttestedPasskey> {
        self.core
            .register_credential(reg, &state.rs, Some(&state.ca_list))
            .map(|cred| AttestedPasskey { cred })
    }

    /// Given a set of `AttestedPasskey`'s, begin an authentication of the user. This returns
    /// a `RequestChallengeResponse`, which should be serialised to json and sent to the user agent (e.g. a browser).
    /// The server must persist the [AttestedPasskeyAuthentication] state as it is paired to the
    /// `RequestChallengeResponse` and required to complete the authentication.
    ///
    /// Finally you need to call [`finish_attested_passkey_authentication`](Webauthn::finish_attested_passkey_authentication)
    /// to complete the authentication.
    ///
    /// WARNING ⚠️  YOU MUST STORE THE [AttestedPasskeyAuthentication] VALUE SERVER SIDE.
    ///
    /// Failure to do so *may* open you to replay attacks which can significantly weaken the
    /// security of this system.
    pub fn start_attested_passkey_authentication(
        &self,
        creds: &[AttestedPasskey],
    ) -> WebauthnResult<(RequestChallengeResponse, AttestedPasskeyAuthentication)> {
        let creds = creds.iter().map(|sk| sk.cred.clone()).collect();

        let extensions = Some(RequestAuthenticationExtensions {
            appid: None,
            uvm: Some(true),
            hmac_get_secret: None,
        });

        let policy = UserVerificationPolicy::Required;
        let allow_backup_eligible_upgrade = false;

        self.core
            .generate_challenge_authenticate_policy(
                creds,
                policy,
                extensions,
                allow_backup_eligible_upgrade,
            )
            .map(|(rcr, ast)| (rcr, AttestedPasskeyAuthentication { ast }))
    }

    /// Given the `PublicKeyCredential` returned by the user agent (e.g. a browser), and the stored [AttestedPasskeyAuthentication]
    /// complete the authentication of the user. This asserts that user verification must have been correctly
    /// performed allowing you to trust this as a MFA interfaction.
    ///
    /// # Errors
    /// If any part of the registration is incorrect or invalid, an error will be returned. See [WebauthnError].
    ///
    /// # Returns
    /// On success, [AuthenticationResult] is returned which contains some details of the Authentication
    /// process.
    ///
    /// As per <https://www.w3.org/TR/webauthn-3/#sctn-verifying-assertion> 21:
    ///
    /// If the Credential Counter is greater than 0 you MUST assert that the counter is greater than
    /// the stored counter. If the counter is equal or less than this MAY indicate a cloned credential
    /// and you SHOULD invalidate and reject that credential as a result.
    ///
    /// From this [AuthenticationResult] you *should* update the Credential's Counter value if it is
    /// valid per the above check. If you wish
    /// you *may* use the content of the [AuthenticationResult] for extended validations (such as the
    /// user verification flag).
    ///
    /// In *some* cases, you *may* be able to identify the user by examinin
    pub fn finish_attested_passkey_authentication(
        &self,
        reg: &PublicKeyCredential,
        state: &AttestedPasskeyAuthentication,
    ) -> WebauthnResult<AuthenticationResult> {
        self.core.authenticate_credential(reg, &state.ast)
    }

    /// WIP DO NOT USE
    pub fn start_discoverable_authentication(
        &self,
    ) -> WebauthnResult<(RequestChallengeResponse, DiscoverableAuthentication)> {
        let policy = UserVerificationPolicy::Required;
        let extensions = Some(RequestAuthenticationExtensions {
            appid: None,
            uvm: Some(true),
            hmac_get_secret: None,
        });

        self.core
            .generate_challenge_authenticate_discoverable(policy, extensions)
            .map(|(rcr, ast)| (rcr, DiscoverableAuthentication { ast }))
    }

    /// WIP DO NOT USE
    pub fn identify_discoverable_authentication<'a>(
        &'_ self,
        reg: &'a PublicKeyCredential,
    ) -> WebauthnResult<(Uuid, &'a [u8])> {
        let cred_id = reg.get_credential_id();
        reg.get_user_unique_id()
            .and_then(|b| Uuid::from_slice(b).ok())
            .map(|u| (u, cred_id))
            .ok_or(WebauthnError::InvalidUserUniqueId)
    }

    /// WIP DO NOT USE
    pub fn finish_discoverable_authentication(
        &self,
        reg: &PublicKeyCredential,
        mut state: DiscoverableAuthentication,
        creds: &[DiscoverableKey],
    ) -> WebauthnResult<AuthenticationResult> {
        let creds = creds.iter().map(|dk| dk.cred.clone()).collect();
        state.ast.set_allowed_credentials(creds);
        self.core.authenticate_credential(reg, &state.ast)
    }
}

#[cfg(feature = "resident-key-support")]
impl Webauthn {
    /// TODO
    pub fn start_attested_resident_key_registration(
        &self,
        user_unique_id: Uuid,
        user_name: &str,
        user_display_name: &str,
        exclude_credentials: Option<Vec<CredentialID>>,
        attestation_ca_list: AttestationCaList,
        ui_hint_authenticator_attachment: Option<AuthenticatorAttachment>,
    ) -> WebauthnResult<(CreationChallengeResponse, AttestedResidentKeyRegistration)> {
        if attestation_ca_list.is_empty() {
            return Err(WebauthnError::MissingAttestationCaList);
        }

        let attestation = AttestationConveyancePreference::Direct;
        let credential_algorithms = self.algorithms.clone();
        let require_resident_key = true;
        let policy = Some(UserVerificationPolicy::Required);
        let reject_passkeys = true;

        // credProtect
        let extensions = Some(RequestRegistrationExtensions {
            cred_protect: Some(CredProtect {
                // Since this will contain PII, we need to enforce this.
                credential_protection_policy: CredentialProtectionPolicy::UserVerificationRequired,
                // Set to true since this function requires attestation, and attestation is really
                // only viable on FIDO2/CTAP2 creds that actually support this.
                enforce_credential_protection_policy: Some(true),
            }),
            // https://www.w3.org/TR/webauthn-2/#sctn-uvm-extension
            uvm: Some(true),
            cred_props: Some(true),
            // https://fidoalliance.org/specs/fido-v2.1-rd-20210309/fido-client-to-authenticator-protocol-v2.1-rd-20210309.html#sctn-minpinlength-extension
            min_pin_length: Some(true),
            hmac_create_secret: Some(true),
        });

        self.core
            .generate_challenge_register_options(
                user_unique_id.as_bytes(),
                user_name,
                user_display_name,
                attestation,
                policy,
                exclude_credentials,
                extensions,
                credential_algorithms,
                require_resident_key,
                ui_hint_authenticator_attachment,
                reject_passkeys,
            )
            .map(|(ccr, rs)| {
                (
                    ccr,
                    AttestedResidentKeyRegistration {
                        rs,
                        ca_list: attestation_ca_list,
                    },
                )
            })
    }

    /// TODO
    pub fn finish_attested_resident_key_registration(
        &self,
        reg: &RegisterPublicKeyCredential,
        state: &AttestedResidentKeyRegistration,
    ) -> WebauthnResult<AttestedResidentKey> {
        let cred = self
            .core
            .register_credential(reg, &state.rs, Some(&state.ca_list))?;

        trace!("finish attested_resident_key -> {:?}", cred);

        // cred protect ignored :(
        // Is the pin long enough?
        // Is it rk?
        // I guess we'll never know ...

        // Is it an approved cred / aaguid?

        Ok(AttestedResidentKey { cred })
    }

    /// TODO
    pub fn start_attested_resident_key_authentication(
        &self,
        creds: &[AttestedResidentKey],
    ) -> WebauthnResult<(RequestChallengeResponse, AttestedResidentKeyAuthentication)> {
        let creds = creds.iter().map(|sk| sk.cred.clone()).collect();
        let extensions = Some(RequestAuthenticationExtensions {
            appid: None,
            uvm: Some(true),
            hmac_get_secret: None,
        });

        let policy = UserVerificationPolicy::Required;
        let allow_backup_eligible_upgrade = false;

        self.core
            .generate_challenge_authenticate_policy(
                creds,
                policy,
                extensions,
                allow_backup_eligible_upgrade,
            )
            .map(|(rcr, ast)| (rcr, AttestedResidentKeyAuthentication { ast }))
    }

    /// TODO
    pub fn finish_attested_resident_key_authentication(
        &self,
        reg: &PublicKeyCredential,
        state: &AttestedResidentKeyAuthentication,
    ) -> WebauthnResult<AuthenticationResult> {
        self.core.authenticate_credential(reg, &state.ast)
    }
}

#[test]
/// Test that building a webauthn object from a chrome extension origin is successful.
fn test_webauthnbuilder_chrome_url() -> Result<(), Box<dyn std::error::Error>> {
    use crate::prelude::*;
    let rp_id = "2114c9f524d0cbd74dbe846a51c3e5b34b83ac02c5220ec5cdff751096fa25a5";
    let rp_origin = Url::parse(&format!("chrome-extension://{rp_id}"))?;
    eprintln!("{rp_origin:?}");
    let builder = WebauthnBuilder::new(rp_id, &rp_origin)?;
    eprintln!("rp_id: {:?}", builder.rp_id);
    let built = builder.build()?;
    eprintln!("rp_name: {}", built.core.rp_name());
    Ok(())
}
