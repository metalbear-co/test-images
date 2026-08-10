import { NestFactory } from '@nestjs/core';
import { ConsoleLogger } from '@nestjs/common';
import { AsyncMicroserviceOptions, Transport } from '@nestjs/microservices';

import { ConfigService } from './config/config.service';
import { InventoryModule } from './inventory/inventory.module';
import { NotificationsModule } from './notifications/notifications.module';

import { AppModule } from './app.module';
import { WorkerModule } from './worker.module';

async function bootstrapApp() {
  const app = await NestFactory.create(AppModule, {
    logger: new ConsoleLogger({ json: true }),
  });

  app.enableShutdownHooks();

  await app.listen(process.env.PORT ?? 3000);
}

async function bootstrapInventoryService() {
  const app = await NestFactory.createMicroservice<AsyncMicroserviceOptions>(
    InventoryModule,
    {
      logger: new ConsoleLogger({ json: true }),
      useFactory: (configService: ConfigService) => configService.inventoryMicroserviceConfig(),
      inject: [ConfigService],
    }
  );

  app.enableShutdownHooks();

  await app.listen();
}

async function bootstrapNotificationsService() {
  const app = await NestFactory.createMicroservice<AsyncMicroserviceOptions>(
    NotificationsModule,
    {
      logger: new ConsoleLogger({ json: true }),
      useFactory: (configService: ConfigService) => configService.notificationsMicroserviceConfig(),
      inject: [ConfigService],
    }
  );

  app.enableShutdownHooks();

  await app.listen();
}

async function bootstrapWorker() {
  const app = await NestFactory.createMicroservice<AsyncMicroserviceOptions>(
    WorkerModule,
    {
      logger: new ConsoleLogger({ json: true }),
      useFactory: (configService: ConfigService) => configService.workerMicroserviceConfig(),
      inject: [ConfigService],
    }
  );

  app.enableShutdownHooks();
  await app.listen();
}

switch(process.env.ENTRYPOINT) {
  case 'inventory-service':
    bootstrapInventoryService();
    break;

  case 'notifications-service':
    bootstrapNotificationsService();
    break;

  case 'worker':
    bootstrapWorker();
    break;

  default:
    bootstrapApp();
}