#!/bin/bash
# Test client auth using cURL and zenity
# Make sure to setup `.env` file with actual data
_ZENITY=$(which zenity)
if [ -z "${_ZENITY}" ]; then
    echo "Please install zenity"
    exit 1
fi

# Load environment variables
source ../.env

# OAuth2 parameters
if [ -z "${GOOGLE_CLIENT_ID}" ]; then
    echo "Please set the GOOGLE_CLIENT_ID environment variable"
    CLIENT_ID="your_client_id"
else
    CLIENT_ID="${GOOGLE_CLIENT_ID}"
fi
if [ -z "${GOOGLE_CLIENT_SECRET}" ]; then
    echo "Please set the GOOGLE_CLIENT_SECRET environment variable "
    CLIENT_SECRET="your_client_secret"
else
    CLIENT_ID="${GOOGLE_CLIENT_SECRET}"
fi
if [ -z "${GOOGLE_REDIRECT_URI}" ]; then
    echo "Please set the GOOGLE_REDIRECT_URI environment variable"
    REDIRECT_URI="your_redirect_uri"
else
    REDIRECT_URI="${GOOGLE_REDIRECT_URI}"
fi
SCOPE="openid profile email"

# Generate authorization URL and prompt user
AUTH_URL="https://accounts.google.com/o/oauth2/auth?client_id=${CLIENT_ID}&redirect_uri=${REDIRECT_URI}&response_type=code&scope=${SCOPE}"

zenity --info --text="Please open your web browser and go to the following URL to authorize the application:\n\n${AUTH_URL}" --title="OAuth2 Authorization"

# Prompt user to enter authorization code
AUTH_CODE=$(zenity --entry --text="Enter the authorization code:" --title="OAuth2 Authorization Code")

# Exchange authorization code for tokens
TOKEN_URL="https://oauth2.googleapis.com/token"

curl -X POST "${TOKEN_URL}" \
     -d "client_id=${CLIENT_ID}" \
     -d "client_secret=${CLIENT_SECRET}" \
     -d "code=${AUTH_CODE}" \
     -d "redirect_uri=${REDIRECT_URI}" \
     -d "grant_type=authorization_code"
