import { Module } from '@nestjs/common';

import { ConfigModule } from '../config/config.module';

import { InventoryController } from './inventory.controller';

@Module({
  imports: [ConfigModule],
  controllers: [InventoryController],
  providers: [],
  exports: []
})
export class InventoryModule {}
