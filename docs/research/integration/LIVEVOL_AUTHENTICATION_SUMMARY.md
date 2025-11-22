# LiveVol Authentication - Quick Summary

**Date**: 2025-01-27
**Source**: [LiveVol API Authentication Docs](https://api.livevol.com/v1/docs/Home/Authentication)

---

## ✅ What You Need

1. **Client ID** - Your application consumer key
2. **Client Secret** - Your application secret key

**Where to Find**:

- LiveVol Pro account → Settings → API → Credentials
- Or contact LiveVol support

---

## 🔐 Authentication Flow

### Endpoint

```
POST https://id.livevol.com/connect/token
```

### Headers

```
Authorization: Basic <base64_encoded_client_id:client_secret>
Content-Type: application/x-www-form-urlencoded
```

### Body

```
grant_type=client_credentials
```

### Response

```json
{
  "access_token": "eyJ0eXAiOi...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

---

## 🧪 Quick Test

### Using the Script

```bash
python scripts/livevol_api_explorer.py \
  --client-id YOUR_CLIENT_ID \
  --client-secret YOUR_CLIENT_SECRET
```

### Using curl

```bash
# Encode credentials
CREDENTIALS=$(echo -n "your_client_id:your_client_secret" | base64)

# Request token
curl -X POST https://id.livevol.com/connect/token \
  -H "Authorization: Basic $CREDENTIALS" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials"
```

---

## 📝 Important Notes

1. **Identity Server**: `https://id.livevol.com` (separate from API server)
2. **Token Endpoint**: `/connect/token` (NOT `/oauth/token`)
3. **Basic Auth**: Use base64 encoded `client_id:client_secret`
4. **Token Expiry**: Access tokens expire in 3600 seconds (1 hour)
5. **API Server**: `https://api.livevol.com/v1` (for API calls after auth)

---

## 📚 Full Documentation

- **Authentication Guide**: <https://api.livevol.com/v1/docs/Home/Authentication>
- **Where to Find Credentials**: `docs/LIVEVOL_CREDENTIALS_WHERE_TO_FIND.md`
- **Exploration Script**: `scripts/livevol_api_explorer.py`

---

**Next Step**: Get your Client ID and Client Secret from LiveVol Pro account, then run the exploration script!
