# LiveVol API Credentials - Where to Find Them

**Date**: 2025-01-27
**Source**: LiveVol API Documentation

---

## Quick Answer

**API Credentials Location**:

1. **API Documentation**: <https://api.livevol.com/v1/docs/Home/Authentication>
2. **LiveVol Pro Platform**: Log in → Settings → API → Credentials
3. **Contact Support**: If not visible, contact LiveVol support

---

## Step-by-Step Instructions

### Method 1: Through LiveVol Pro Platform

1. **Log into LiveVol Pro**
   - URL: <https://datashop.cboe.com/livevol-pro>
   - Or: <https://www.livevol.com>
   - Use your trial account credentials

2. **Navigate to API Settings**
   - Look for **"Settings"** or **"Account Settings"**
   - Go to **"API"** or **"Developer"** section
   - Find **"API Credentials"** or **"API Keys"**

3. **Get Your Credentials**
   - **Client ID**: OAuth 2.0 client identifier
   - **Client Secret**: OAuth 2.0 client secret
   - **Base URL**: Usually `https://api.livevol.com/v1`

### Method 2: Through API Documentation

1. **Access API Documentation**
   - URL: <https://api.livevol.com/v1/docs/>
   - Or: <https://api.livevol.com/v1/docs/Home/Authentication>

2. **Check Authentication Section**
   - Look for **"Authentication"** or **"Getting Started"**
   - Find instructions for obtaining credentials
   - May require account setup or registration

3. **Follow Authentication Guide**
   - Review OAuth 2.0 setup instructions
   - May need to register your application
   - Get Client ID and Client Secret

### Method 3: Contact LiveVol Support

If credentials are not visible:

1. **Contact Support**
   - Phone: Check LiveVol website
   - Email: Support contact form
   - Through trial account: Look for "Support" or "Help"

2. **Request API Access**
   - Mention you have a LiveVol Pro trial
   - Request API credentials (Client ID, Client Secret)
   - Ask about API access for trial accounts

---

## What You Need

### Required Credentials

1. **Client ID** (OAuth 2.0)
   - Example format: `your_client_id_here`
   - Used for API authentication

2. **Client Secret** (OAuth 2.0)
   - Example format: `your_client_secret_here`
   - Keep this secret - never commit to git

3. **Base URL** (Optional)
   - Default: `https://api.livevol.com/v1`
   - May vary for different environments

### Authentication Method

LiveVol uses **OAuth 2.0 Client Credentials** flow:

```
POST https://api.livevol.com/v1/oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
&client_id=YOUR_CLIENT_ID
&client_secret=YOUR_CLIENT_SECRET
```

Returns:

```json
{
  "access_token": "your_access_token",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

---

## Common Locations in LiveVol Pro

### Option 1: Settings Menu

- **Settings** → **API** → **Credentials**
- **Account** → **API Keys**
- **Developer** → **API Access**

### Option 2: User Profile

- **Profile** → **API Settings**
- **Account** → **API Credentials**
- **My Account** → **API Access**

### Option 3: Documentation Links

- **Help** → **API Documentation**
- **Support** → **API Access**
- **Resources** → **API Guide**

---

## Trial Account Considerations

### API Access in Trial

**Questions to Check**:

- [ ] Does trial include API access?
- [ ] Are API credentials automatically provided?
- [ ] Do I need to request API access separately?
- [ ] Are there limitations on API usage in trial?

### If API Access Not Available

**Alternatives**:

1. **Contact Support** - Request API access for trial
2. **Check Documentation** - May have trial-specific setup
3. **Use Web Interface** - Explore features via web platform
4. **Export Data** - Check if web platform allows data export

---

## Verification Steps

Once you have credentials:

1. **Test Authentication**

   ```bash
   curl -X POST https://api.livevol.com/v1/oauth/token \
     -d "grant_type=client_credentials" \
     -d "client_id=YOUR_CLIENT_ID" \
     -d "client_secret=YOUR_CLIENT_SECRET"
   ```

2. **Check Response**
   - Should return `access_token`
   - Status code should be `200 OK`
   - If `401 Unauthorized`, check credentials

3. **Test API Endpoint**

   ```bash
   curl -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
     https://api.livevol.com/v1/...
   ```

---

## Troubleshooting

### Issue 1: Can't Find API Settings

**Symptoms**: No API section in settings

**Solutions**:

- ✅ Check if trial includes API access
- ✅ Look in different menu locations
- ✅ Contact LiveVol support
- ✅ Check API documentation for setup instructions

### Issue 2: Credentials Not Provided

**Symptoms**: No Client ID/Secret visible

**Solutions**:

- ✅ Request API access from support
- ✅ Check if API access requires separate subscription
- ✅ Verify trial account type includes API access
- ✅ Check email for API credentials

### Issue 3: Authentication Fails

**Symptoms**: `401 Unauthorized` or `403 Forbidden`

**Solutions**:

- ✅ Verify Client ID and Secret are correct
- ✅ Check if credentials are for API (not web login)
- ✅ Verify OAuth endpoint URL
- ✅ Check if trial includes API access
- ✅ Contact support for credential verification

---

## Next Steps

1. **Get Credentials** - Follow steps above to obtain Client ID and Secret
2. **Test Authentication** - Verify credentials work
3. **Run Exploration Script** - Test API endpoints
4. **Document Findings** - Record what's available

---

## Resources

- **LiveVol API Docs**: <https://api.livevol.com/v1/docs/>
- **Authentication Guide**: <https://api.livevol.com/v1/docs/Home/Authentication>
- **LiveVol Pro**: <https://datashop.cboe.com/livevol-pro>
- **User Guide**: <https://www.livevol.com/user-guide/>

---

## Quick Checklist

- [ ] Logged into LiveVol Pro
- [ ] Checked Settings → API section
- [ ] Checked API Documentation
- [ ] Contacted support (if needed)
- [ ] Obtained Client ID
- [ ] Obtained Client Secret
- [ ] Tested authentication
- [ ] Ready to run exploration script

---

**Once you have credentials, run:**

```bash
python scripts/livevol_api_explorer.py \
  --client-id YOUR_CLIENT_ID \
  --client-secret YOUR_CLIENT_SECRET
```
