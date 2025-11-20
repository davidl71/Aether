# LiveVol API Credentials - Where to Find Them

**Date**: 2025-01-27
**Source**: [LiveVol API Authentication Documentation](https://api.livevol.com/v1/docs/Home/Authentication)

---

## ✅ Authentication Method

LiveVol uses **OAuth 2.0 Client Credentials Flow** for machine-to-machine communication.

**Key Details**:
- **Token Endpoint**: `https://id.livevol.com/connect/token`
- **Authentication**: Basic Auth with base64 encoded `client_id:client_secret`
- **Grant Type**: `client_credentials`
- **Response**: Returns `access_token`, `expires_in` (3600s), `token_type` (Bearer)

---

## 📍 Where to Find Your Credentials

### Option 1: LiveVol Pro Account Settings

1. **Log into LiveVol Pro**
   - URL: <https://datashop.cboe.com/livevol-pro>
   - Or: <https://www.livevol.com>
   - Use your trial account credentials

2. **Navigate to API Settings**
   - Look for **"Settings"** → **"API"** → **"Credentials"**
   - Or: **"Account"** → **"API Keys"**
   - Or: **"Developer"** → **"API Access"**
   - Or: **"Profile"** → **"API Settings"**

3. **Get Your Credentials**
   - **Client ID**: Your application consumer key
   - **Client Secret**: Your application secret key
   - Keep these secure - never commit to git

### Option 2: Contact LiveVol Support

If credentials are not visible in your account:

1. **Contact Support**
   - Phone: Check LiveVol website
   - Email: Support contact form
   - Through trial account: Look for "Support" or "Help"

2. **Request API Credentials**
   - Mention you have a LiveVol Pro trial
   - Request Client ID and Client Secret for API access
   - Ask about API access for trial accounts

### Option 3: Check Welcome Email

1. **Check Your Email**
   - Look for LiveVol Pro trial welcome email
   - May contain API credentials or setup instructions
   - Check spam folder if not in inbox

2. **Check Account Dashboard**
   - Look for "API Access" or "Developer" section
   - May have setup instructions or credential generation

---

## 🔐 Authentication Flow

### Step 1: Get Credentials

You need:
- **Client ID**: Example format: `your_client_id_here`
- **Client Secret**: Example format: `your_client_secret_here`

### Step 2: Request Access Token

**Endpoint**: `https://id.livevol.com/connect/token`

**Method**: `POST`

**Headers**:
```
Authorization: Basic <base64_encoded_client_id:client_secret>
Content-Type: application/x-www-form-urlencoded
```

**Body**:
```
grant_type=client_credentials
```

**Example Request**:
```bash
curl -X POST https://id.livevol.com/connect/token \
  -H "Authorization: Basic $(echo -n 'your_client_id:your_client_secret' | base64)" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials"
```

**Example Response**:
```json
{
  "access_token": "eyJ0eXAiOi...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

### Step 3: Use Access Token

**Header**:
```
Authorization: Bearer <access_token>
```

**Example Request**:
```bash
curl -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  https://api.livevol.com/v1/market/symbols
```

---

## 🧪 Quick Test

### Test Authentication

```bash
# Set your credentials
export LIVEVOL_CLIENT_ID="your_client_id"
export LIVEVOL_CLIENT_SECRET="your_client_secret"

# Test authentication
python scripts/livevol_api_explorer.py \
  --client-id "$LIVEVOL_CLIENT_ID" \
  --client-secret "$LIVEVOL_CLIENT_SECRET"
```

### Manual Test with curl

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

## ✅ Verification Checklist

- [ ] Logged into LiveVol Pro account
- [ ] Checked Settings → API section
- [ ] Checked Account → API Keys
- [ ] Checked Developer → API Access
- [ ] Checked welcome email
- [ ] Contacted support (if needed)
- [ ] Obtained Client ID
- [ ] Obtained Client Secret
- [ ] Tested authentication
- [ ] Received access token

---

## 🚨 Troubleshooting

### Issue 1: Can't Find Credentials

**Symptoms**: No API section in settings

**Solutions**:
- ✅ Check if trial includes API access
- ✅ Look in different menu locations
- ✅ Contact LiveVol support
- ✅ Check welcome email
- ✅ Check account dashboard

### Issue 2: Authentication Fails (401 Unauthorized)

**Symptoms**: `401 Unauthorized` or `403 Forbidden`

**Solutions**:
- ✅ Verify Client ID and Secret are correct
- ✅ Check if credentials are base64 encoded correctly
- ✅ Verify endpoint: `https://id.livevol.com/connect/token`
- ✅ Check if trial includes API access
- ✅ Verify grant_type is `client_credentials`
- ✅ Check Content-Type header is set

### Issue 3: Wrong Endpoint

**Symptoms**: `404 Not Found`

**Solutions**:
- ✅ Use correct endpoint: `https://id.livevol.com/connect/token`
- ✅ NOT: `https://api.livevol.com/v1/oauth/token`
- ✅ Identity server is separate from API server

---

## 📚 Resources

- **Authentication Docs**: <https://api.livevol.com/v1/docs/Home/Authentication>
- **API Documentation**: <https://api.livevol.com/v1/docs/>
- **LiveVol Pro**: <https://datashop.cboe.com/livevol-pro>
- **User Guide**: <https://www.livevol.com/user-guide/>

---

## 🔄 Next Steps

1. **Get Credentials** - Follow steps above
2. **Test Authentication** - Use the exploration script
3. **Explore API** - Test endpoints for quoted spreads
4. **Document Findings** - Record what's available

---

## 📝 Important Notes

1. **Token Expiration**: Access tokens expire in 3600 seconds (1 hour)
2. **Token Refresh**: Use refresh token to get new access token
3. **Security**: Never commit credentials to git
4. **Trial Access**: Verify trial includes API access
5. **Rate Limits**: Check API docs for rate limits

---

**Once you have credentials, run:**
```bash
python scripts/livevol_api_explorer.py \
  --client-id YOUR_CLIENT_ID \
  --client-secret YOUR_CLIENT_SECRET
```
