## iOS Certificate Pinning

The iPad companion app now enforces App Transport Security (ATS) pinning through the `NSAppTransportSecurity.NSPinnedDomains` configuration. Run-time
networking will fail unless the pinned information matches the certificate chain that secures your API endpoint. Follow the steps below to update the
placeholder values introduced in `ios/BoxSpreadIPad/Info.plist`.

1. **Identify the production hostname.**
Replace `api.boxspread.example` with the exact host your app contacts (for example, `api.ibboxspread.yourdomain.com`). Leave `NSIncludesSubdomains`
set to `true` if you talk to subdomains; otherwise set it to `false`.

2. **Export the leaf certificate’s SPKI hash.**
   Fetch the remote certificate and convert it to a base64-encoded SHA-256 hash of the Subject Public Key Info (SPKI). You can do this with OpenSSL:

   ```bash
   HOST=api.yourdomain.com
   PORT=443
   openssl s_client -connect "${HOST}:${PORT}" -servername "${HOST}" < /dev/null \
     | openssl x509 -pubkey -noout \
     | openssl pkey -pubin -outform DER \
     | openssl dgst -sha256 -binary \
     | openssl base64
   ```

   The command prints the value to copy into `SPKI-SHA256-BASE64`. Repeat the process whenever the leaf certificate changes.

3. **(Optional) Add backup pins.**
To support certificate rotation, add additional `<dict>` entries to `NSPinnedLeafIdentities` with the upcoming certificate’s SPKI, or include
`NSPinnedCAIdentities` if you pin to an intermediate/issuing CA.

4. **Require certificate transparency if available.**
`NSRequiresCertificateTransparency` is set to `true`, aligning with Apple’s guidance for production endpoints. Set it to `false` only if your CA does
not currently supply SCTs.

5. **Bundle DER certificate for offline validation (optional).**
If you also perform manual pin checks in Swift, add the DER file to the app bundle’s resources to keep the certificate hash aligned with the ATS
configuration.

After updating the plist and bundling any needed assets, rebuild the app to ensure connectivity succeeds. When rotating certificates, remember to
update the hash before the old certificate expires so users don’t experience downtime.
