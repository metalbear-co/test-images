import { Module } from '@nestjs/common';

import { ConfigModule } from '../config/config.module';

import { NotificationsController } from './notifications.controller';

@Module({
  imports: [ConfigModule],
  controllers: [NotificationsController],
  providers: [],
  exports: []
})
export class NotificationsModule {}
