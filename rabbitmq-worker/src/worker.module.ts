import { Module } from '@nestjs/common';

import { ConfigModule } from './config/config.module'
import { InventoryModule } from './inventory/inventory.module'
import { NotificationsModule } from './notifications/notifications.module'

@Module({
  imports: [ConfigModule, InventoryModule, NotificationsModule],
  controllers: [],
  providers: [],
})
export class WorkerModule {}
