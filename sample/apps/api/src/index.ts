import express from 'express';
import cors from 'cors';
import { createLogger } from '@sample/logger';
import { loadConfig } from '@sample/config';
import type { User, ApiResponse } from '@sample/types';

const config = loadConfig();
const logger = createLogger({
  level: config.logLevel,
  service: 'api'
});

const app = express();

app.use(cors());
app.use(express.json());

app.get('/users', (req, res) => {
  const users: User[] = [
    { id: '1', email: 'admin@example.com', name: 'Admin', role: 'admin' }
  ];
  
  const response: ApiResponse<User[]> = {
    data: users,
    status: 200
  };
  
  res.json(response);
});

app.listen(config.port, () => {
  logger.info(`API server started on port ${config.port}`);
}); 