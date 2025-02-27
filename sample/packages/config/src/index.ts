import { config } from 'dotenv';

export interface AppConfig {
  port: number;
  environment: string;
  logLevel: string;
}

export function loadConfig(): AppConfig {
  config();
  
  return {
    port: parseInt(process.env.PORT || '3000', 10),
    environment: process.env.NODE_ENV || 'development',
    logLevel: process.env.LOG_LEVEL || 'info'
  };
} 