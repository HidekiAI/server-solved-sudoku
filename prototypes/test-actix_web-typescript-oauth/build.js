const fs = require('fs');
const dotenv = require('dotenv');

// Load environment variables from .env file
dotenv.config();

// Read the template TypeScript file
const scriptTs = fs.readFileSync('script.ts', 'utf8');

// Replace placeholders with environment variables
const scriptJs = scriptTs
  .replace('GOOGLE_CLIENT_ID_FROM_SWAPPED_IN_BUILD', process.env.GOOGLE_CLIENT_ID)
  .replace('GOOGLE_CLIENT_SECRET_SWAPPED_IN_BUILD', process.env.GOOGLE_CLIENT_SECRET)
  .replace('GOOGLE_REDIRECT_URI_SWAPPED_IN_BUILD', process.env.GOOGLE_REDIRECT_URI);

// Write the resulting JavaScript file
fs.writeFileSync('script.js', scriptJs, 'utf8');

console.log('script.js generated successfully.');
