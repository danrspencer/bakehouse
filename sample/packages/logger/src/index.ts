import winston from 'winston';

export interface LoggerConfig {
  level: string;
  service: string;
}

export function createLogger(config: LoggerConfig) {
  return winston.createLogger({
    level: config.level,
    defaultMeta: { service: config.service },
    transports: [
      new winston.transports.Console({
        format: winston.format.combine(
          winston.format.timestamp(),
          winston.format.json()
        )
      })
    ]
  });
} 