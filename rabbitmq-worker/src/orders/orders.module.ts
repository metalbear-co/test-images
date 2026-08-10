import { Module } from '@nestjs/common';
import { ClientsModule } from '@nestjs/microservices';
import { ServeStaticModule } from '@nestjs/serve-static';

import * as path from 'node:path';

import { ConfigModule } from '../config/config.module';
import { ConfigService } from '../config/config.service';

import { OrdersController } from './orders.controller';

@Module({
  imports: [
    ClientsModule.registerAsync([
      { 
        imports: [ConfigModule],
        name: 'WORKER_SERVICE',
        useFactory: (configService: ConfigService) => configService.rabbitMQClientConfig(),
        inject: [ConfigService],
      },
    ]),
  ],
  controllers: [OrdersController],
  providers: [],
})
export class OrdersModule {}
