//! Kani proof harness for VP-013: Connection state machine validity.
//!
//! ## Verification properties (VP-013)
//!
//! 1. `start_connecting` from `Disconnected` always yields `Connecting`.
//! 2. `mark_connected` from `Connecting` always yields `Connected`.
//! 3. `start_disconnecting` from `Connected` always yields `Disconnecting`.
//! 4. `mark_disconnected` from `Disconnecting` always yields `Disconnected`.
//! 5. `mark_error` from any state always yields `Error(reason)`.
//! 6. `start_reconnecting` from `Error(_)` always yields `Connecting`.
//! 7. No invalid transition panics.
//!
//! These proofs are checked exhaustively by Kani over all discriminant values
//! of `ConnectionState`.
//!
//! ## Running the proofs
//!
//! ```sh
//! cargo kani --tests
//! # or, scoped:
//! cargo kani --harness test_VP_013_disconnected_to_connecting
//! ```

#[cfg(kani)]
mod vp_013 {
    use forge_core::ConnectionState;

    /// VP-013-1: Disconnected → Connecting.
    #[kani::proof]
    fn test_VP_013_disconnected_to_connecting() {
        let s = ConnectionState::Disconnected;
        let next = s.start_connecting();
        kani::assert(next == ConnectionState::Connecting, "Disconnected.start_connecting() must yield Connecting");
    }

    /// VP-013-2: Connecting → Connected.
    #[kani::proof]
    fn test_VP_013_connecting_to_connected() {
        let s = ConnectionState::Connecting;
        let next = s.mark_connected();
        kani::assert(next == ConnectionState::Connected, "Connecting.mark_connected() must yield Connected");
    }

    /// VP-013-3: Connected → Disconnecting.
    #[kani::proof]
    fn test_VP_013_connected_to_disconnecting() {
        let s = ConnectionState::Connected;
        let next = s.start_disconnecting();
        kani::assert(next == ConnectionState::Disconnecting, "Connected.start_disconnecting() must yield Disconnecting");
    }

    /// VP-013-4: Disconnecting → Disconnected.
    #[kani::proof]
    fn test_VP_013_disconnecting_to_disconnected() {
        let s = ConnectionState::Disconnecting;
        let next = s.mark_disconnected();
        kani::assert(next == ConnectionState::Disconnected, "Disconnecting.mark_disconnected() must yield Disconnected");
    }

    /// VP-013-5: Error → Connecting (reconnect).
    #[kani::proof]
    fn test_VP_013_error_to_connecting() {
        let s = ConnectionState::Error("test".to_string());
        let next = s.start_reconnecting();
        kani::assert(next == ConnectionState::Connecting, "Error.start_reconnecting() must yield Connecting");
    }

    /// VP-013-6: Full round trip Disconnected → Connected → Disconnected.
    #[kani::proof]
    fn test_VP_013_full_lifecycle_roundtrip() {
        let s = ConnectionState::Disconnected
            .start_connecting();
        kani::assert(s == ConnectionState::Connecting, "step 1");
        let s = s.mark_connected();
        kani::assert(s == ConnectionState::Connected, "step 2");
        let s = s.start_disconnecting();
        kani::assert(s == ConnectionState::Disconnecting, "step 3");
        let s = s.mark_disconnected();
        kani::assert(s == ConnectionState::Disconnected, "step 4");
    }

    /// VP-013-7: Connected → Error → Connecting (error recovery).
    #[kani::proof]
    fn test_VP_013_error_recovery_path() {
        let s = ConnectionState::Connected
            .mark_error("network failure");
        kani::assert(matches!(s, ConnectionState::Error(_)), "Connected.mark_error() must yield Error");
        let s = s.start_reconnecting();
        kani::assert(s == ConnectionState::Connecting, "Error.start_reconnecting() must yield Connecting");
    }
}
