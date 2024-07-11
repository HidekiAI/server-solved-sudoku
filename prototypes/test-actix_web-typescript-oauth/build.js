const fs = require('fs');
const dotenv = require('dotenv');

// Load environment variables from .env file
dotenv.config();

// Read the template TypeScript file
const scriptTs = fs.readFileSync('script.ts', 'utf8');

// Replace placeholders with environment variables
const scriptJs = scriptTs
  .replace('YOUR_CLIENT_ID', process.env.GOOGLE_CLIENT_ID)
  .replace('YOUR_CLIENT_SECRET', process.env.GOOGLE_CLIENT_SECRET);

// Write the resulting JavaScript file
fs.writeFileSync('script.js', scriptJs, 'utf8');

console.log('script.js generated successfully.');
