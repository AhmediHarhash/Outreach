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
    console.log('Connected successfully!');

    // Read migration file
    const migrationPath = path.join(__dirname, '..', 'src', 'db', 'migrations', '001_init.sql');
    const sql = fs.readFileSync(migrationPath, 'utf8');

    console.log('Running migration (this may take a moment)...');
    await client.query(sql);

    console.log('Migration completed successfully!');

    // Verify tables created
    const result = await client.query(`
      SELECT table_name
      FROM information_schema.tables
      WHERE table_schema = 'public'
      ORDER BY table_name
    `);

    console.log('\nTables created:');
    result.rows.forEach(row => console.log('  - ' + row.table_name));

  } catch (error) {
    console.error('Migration failed:', error.message);
    if (error.position) {
      console.error('Error at position:', error.position);
    }
    process.exit(1);
  } finally {
    await client.end();
  }
}

runMigration();
