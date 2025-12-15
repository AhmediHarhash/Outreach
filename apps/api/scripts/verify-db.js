const { Client } = require('pg');

const DATABASE_URL = 'postgresql://neondb_owner:npg_xERzTb5M3FrX@ep-shiny-smoke-ah5u9bkl-pooler.c-3.us-east-1.aws.neon.tech/neondb?sslmode=require';

async function verify() {
  const client = new Client({
    connectionString: DATABASE_URL,
    ssl: { rejectUnauthorized: false }
  });

  try {
    await client.connect();

    // Check extensions
    const extensions = await client.query(`
      SELECT extname, extversion FROM pg_extension
      WHERE extname IN ('uuid-ossp', 'pgcrypto', 'vector')
    `);
    console.log('Extensions:');
    extensions.rows.forEach(r => console.log(`  - ${r.extname} v${r.extversion}`));

    // Check subscription plans
    const plans = await client.query('SELECT id, name, price_monthly_usd FROM subscription_plans ORDER BY sort_order');
    console.log('\nSubscription Plans:');
    plans.rows.forEach(r => console.log(`  - ${r.id}: ${r.name} ($${r.price_monthly_usd}/mo)`));

    // Check locales
    const locales = await client.query('SELECT code, native_name, direction FROM supported_locales ORDER BY sort_order');
    console.log('\nSupported Locales:');
    locales.rows.forEach(r => console.log(`  - ${r.code}: ${r.native_name} (${r.direction})`));

    console.log('\n✅ Database ready for production!');

  } catch (error) {
    console.error('Error:', error.message);
  } finally {
    await client.end();
  }
}

verify();
