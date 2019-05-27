initSidebarItems({"enum":[["InternalError",""]],"fn":[["calculate_signature","Calculate the signature for a message."],["create_hkdf","Create a new HMAC-based key derivation function."],["generate_identity_key_pair","A helper function for generating a new [`IdentityKeyPair`]."],["generate_key_pair","Generate a normal elliptic curve key pair."],["generate_pre_keys","Generate a list of [`PreKey`]s. Clients should do this at install time, and subsequently any time the list of [`PreKey`]s stored on the server runs low."],["generate_registration_id","Generate a new registration ID."],["generate_signed_pre_key","Generate a signed pre-key."],["session_builder","Create a new session builder for communication with the user with the specified address."],["store_context","Create a container for the state used by the signal protocol."]],"mod":[["crypto","Underlying cryptographic routines."],["keys","Elliptic curve cryptography keys."]],"struct":[["Address","A reference-counted pointer to a signal address (recipient name, device ID tuple)."],["Buffer","A byte buffer (e.g. `Vec<u8>`)."],["Context","Global state and callbacks used by the library."],["HMACBasedKeyDerivationFunction","Context for a HMAC-based Key Derivation Function."],["PreKeyBundle","The session state used when sending a message to another user."],["PreKeyBundleBuilder","A builder type for the [`PreKeyBundle`]."],["SerializedSession","A serialized session."],["SessionBuilder","Create a new session."],["SessionCipher","The cipher context used for encryption."],["StoreContext","Something which contains state used by the signal protocol."]],"trait":[["FromInternalErrorCode","A helper trait for going from an [`InternalError`] to a `Result`."],["IdentityKeyStore","Something used to store identity keys and track trusted identities."],["IntoInternalErrorCode","A helper trait for going from a `Result` to an [`InternalError`]."],["PreKeyStore","Something which can store [`crate::keys::PreKey`]s without inspecting their contents."],["Serializable","A helper trait for something which can be serialized to protobufs."],["SessionStore","Something which can store the sessions established with recipients."],["SignedPreKeyStore","Something which can store signed pre-keys without inspecting their contents."]]});