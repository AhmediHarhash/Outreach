const { Client } = require('pg');
const fs = require('fs');
const path = require('path');

const DATABASE_URL = 'postgresql://neondb_owner:npg_xERzTb5M3FrX@ep-shiny-smoke-ah5u9bkl-pooler.c-3.us-east-1.aws.neon.tech/neondb?sslmode=require';

async function runMigration() {
  const client = new Client({
    connectionString: DATABASE_URL,
    ssl: { rejectUnauthorized: false }
  });

  try {
    console.log('Connecting to Neon database...');
    await client.connect();
    console.log('Connected!');

    const migrationPath = path.join(__dirname, '..', 'src', 'db', 'migrations', '002_email_system.sql');
    const sql = fs.readFileSync(migrationPath, 'utf8');

    console.log('Running email system migration...');
    await client.query(sql);

    console.log('Migration 002 completed!');

    // Verify
    const templates = await client.query('SELECT id, name FROM email_templates');
    console.log('\nEmail templates created:');
    templates.rows.forEach(r => console.log(`  - ${r.id}: ${r.name}`));

    const tables = await client.query(`
      SELECT table_name FROM information_schema.tables
      WHERE table_schema = 'public' AND table_name LIKE 'email%' OR table_name LIKE 'password%'
      ORDER BY table_name
    `);
    console.log('\nNew tables:');
    tables.rows.forEach(r => console.log(`  - ${r.table_name}`));

  } catch (error) {
    console.error('Migration failed:', error.message);
    process.exit(1);
  } finally {
    await client.end();
  }
}

runMigration();
