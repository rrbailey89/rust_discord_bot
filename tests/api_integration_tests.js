// API Integration Tests
const axios = require('axios');
const { expect } = require('chai');

// Configuration
const API_URL = process.env.API_URL || 'http://localhost:3000/api';
let authToken = null;
let testGuildId = null;

// Utility function to make authenticated requests
async function authenticatedRequest(method, endpoint, data = null) {
  const options = {
    method,
    url: `${API_URL}${endpoint}`,
    headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
    validateStatus: () => true, // Don't throw on error status codes
  };
  
  if (data) {
    options.data = data;
  }
  
  return axios(options);
}

describe('API Integration Tests', function() {
  this.timeout(10000); // Extend timeout for API calls
  
  // Before all tests, try to authenticate
  before(async function() {
    console.log('⚠️ NOTE: These tests require a development instance of the API server running');
    console.log('⚠️ Some tests require valid Discord credentials and will be skipped if not available');
    
    // Check if API is running
    try {
      const response = await axios.get(`${API_URL}/health`);
      if (response.status !== 200) {
        this.skip();
      }
      console.log('✅ API server is running');
    } catch (error) {
      console.error('❌ API server is not running, skipping tests');
      this.skip();
    }
    
    // If TEST_AUTH_TOKEN is provided in environment, use it
    if (process.env.TEST_AUTH_TOKEN) {
      authToken = process.env.TEST_AUTH_TOKEN;
      console.log('✅ Using provided authentication token');
    } else {
      console.log('⚠️ No authentication token provided, some tests will be skipped');
    }
    
    // If TEST_GUILD_ID is provided in environment, use it
    if (process.env.TEST_GUILD_ID) {
      testGuildId = process.env.TEST_GUILD_ID;
      console.log(`✅ Using provided guild ID: ${testGuildId}`);
    } else {
      console.log('⚠️ No guild ID provided, some tests will be skipped');
    }
  });
  
  // Public endpoints
  describe('Public Endpoints', function() {
    it('should return health status', async function() {
      const response = await axios.get(`${API_URL}/health`);
      expect(response.status).to.equal(200);
      expect(response.data).to.have.property('status', 'ok');
    });
    
    it('should return version information', async function() {
      const response = await axios.get(`${API_URL}/version`);
      expect(response.status).to.equal(200);
      expect(response.data).to.have.property('version');
      expect(response.data).to.have.property('name');
    });
  });
  
  // Authentication
  describe('Authentication', function() {
    it('should reject unauthorized requests to protected endpoints', async function() {
      const response = await axios.get(`${API_URL}/guilds`, { validateStatus: () => true });
      expect(response.status).to.equal(401);
    });
    
    // Skip actual OAuth tests as they require browser interaction
    it('should authenticate with Discord OAuth (manual test)');
    
    // If we have a token, test token validation
    if (authToken) {
      it('should validate a legitimate token', async function() {
        const response = await authenticatedRequest('get', '/auth/validate');
        expect(response.status).to.equal(200);
        expect(response.data).to.have.property('valid', true);
      });
    }
  });
  
  // Guild endpoints - requires authentication and guild access
  if (authToken && testGuildId) {
    describe('Guild Management', function() {
      it('should list guilds the user has access to', async function() {
        const response = await authenticatedRequest('get', '/guilds');
        expect(response.status).to.equal(200);
        expect(response.data).to.be.an('array');
        expect(response.data.length).to.be.at.least(1);
      });
      
      it('should get detailed information about a specific guild', async function() {
        const response = await authenticatedRequest('get', `/guilds/${testGuildId}`);
        expect(response.status).to.equal(200);
        expect(response.data).to.have.property('id', testGuildId);
        expect(response.data).to.have.property('name');
      });
      
      it('should retrieve guild settings', async function() {
        const response = await authenticatedRequest('get', `/guilds/${testGuildId}/settings`);
        expect(response.status).to.equal(200);
      });
      
      // Note: We don't test setting updates to avoid changing production settings
      it('should update guild settings (skipped to avoid modifying settings)');
    });
    
    describe('Command Configuration', function() {
      it('should list available commands for a guild', async function() {
        const response = await authenticatedRequest('get', `/guilds/${testGuildId}/commands`);
        expect(response.status).to.equal(200);
        expect(response.data).to.be.an('array');
      });
      
      // Note: We don't test command updates to avoid changing production settings
      it('should update command configuration (skipped to avoid modifying configuration)');
    });
    
    describe('Word Detection', function() {
      it('should list word detection rules', async function() {
        const response = await authenticatedRequest('get', `/guilds/${testGuildId}/word-detection`);
        expect(response.status).to.equal(200);
        expect(response.data).to.be.an('array');
      });
      
      it('should test a word detection pattern', async function() {
        const testData = {
          pattern: 'test_pattern',
          text: 'This contains a test_pattern to match'
        };
        
        const response = await authenticatedRequest('post', `/guilds/${testGuildId}/word-detection/test`, testData);
        expect(response.status).to.equal(200);
        expect(response.data).to.have.property('matches', true);
      });
      
      // Note: We don't test rule creation/updates to avoid changing production rules
      it('should create, update, and delete rules (skipped to avoid modifying rules)');
    });
  }
  
  // Analytics endpoints - requires authentication
  if (authToken) {
    describe('Analytics', function() {
      it('should retrieve analytics summary', async function() {
        const response = await authenticatedRequest('get', '/analytics/summary');
        expect(response.status).to.equal(200);
        expect(response.data).to.be.an('array');
      });
      
      it('should retrieve analytics data', async function() {
        const response = await authenticatedRequest('get', '/analytics/data');
        expect(response.status).to.equal(200);
        expect(response.data).to.be.an('object');
        expect(response.data).to.have.property('commandUsage');
        expect(response.data).to.have.property('userActivity');
      });
      
      it('should filter analytics data by guild', async function() {
        if (!testGuildId) {
          this.skip();
          return;
        }
        
        const response = await authenticatedRequest('get', `/analytics/data?guildId=${testGuildId}`);
        expect(response.status).to.equal(200);
        expect(response.data).to.be.an('object');
      });
      
      it('should filter analytics data by date range', async function() {
        const startDate = new Date();
        startDate.setDate(startDate.getDate() - 7); // 7 days ago
        
        const endDate = new Date();
        
        const response = await authenticatedRequest(
          'get', 
          `/analytics/data?startDate=${startDate.toISOString()}&endDate=${endDate.toISOString()}`
        );
        expect(response.status).to.equal(200);
        expect(response.data).to.be.an('object');
      });
    });
  }
  
  // Error handling
  describe('Error Handling', function() {
    it('should return 404 for non-existent endpoints', async function() {
      const response = await axios.get(`${API_URL}/non-existent-endpoint`, { validateStatus: () => true });
      expect(response.status).to.equal(404);
    });
    
    it('should return 400 for invalid parameters', async function() {
      if (!authToken || !testGuildId) {
        this.skip();
        return;
      }
      
      const response = await authenticatedRequest('post', `/guilds/${testGuildId}/word-detection/test`, {
        // Missing required parameters
      });
      expect(response.status).to.equal(400);
    });
    
    // Test rate limiting if possible
    it('should enforce rate limits (manual test)');
  });
});
