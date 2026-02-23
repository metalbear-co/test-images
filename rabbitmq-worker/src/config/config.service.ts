import { Injectable } from '@nestjs/common';
import { ConfigService as NestConfigService } from '@nestjs/config';
import { ClientProvider, Transport, MicroserviceOptions } from '@nestjs/microservices';

type PartialRMQClientProviderOptions = Partial<(ClientProvider & { transport: Transport.RMQ })['options']>;
type PartialRMQMicroserviceOptionsOptions = Partial<(MicroserviceOptions & { transport: Transport.RMQ })['options']>;

@Injectable()
export class ConfigService {
  constructor(private configService: NestConfigService) {}

  rabbitMQClientConfig(options: PartialRMQClientProviderOptions = {}): ClientProvider {
    return {
      transport: Transport.RMQ,
      options: {
        urls: [this.configService.get('RABBIT_MQ_URL')],
        exchange: this.configService.get('RABBIT_MQ_EXCHANGE'),
        exchangeType: 'topic',
        wildcards: true,
        ...options
      }
    };
  }

  inventoryMicroserviceConfig(): MicroserviceOptions {
    return this.rabbitMQMicroserviceConfig({ 
      queue: this.configService.get('RABBIT_MQ_INVENTORY_QUEUE', 'inventory'),
    })
  }

  notificationsMicroserviceConfig(): MicroserviceOptions {
    return this.rabbitMQMicroserviceConfig({
      queue: this.configService.get('RABBIT_MQ_ORDERS_QUEUE', 'orders'),
      queueOptions: {
        arguments: {
          'x-queue-type': 'quorum'
        },
        durable: true
      }
    })
  }

  workerMicroserviceConfig(): MicroserviceOptions {
    return this.rabbitMQMicroserviceConfig({
      queueOptions: {
        durable: false
      }
    })
  }

  rabbitMQMicroserviceConfig(options: PartialRMQMicroserviceOptionsOptions = {}): MicroserviceOptions {
    return {
      transport: Transport.RMQ,
      options: {
        urls: [this.configService.get('RABBIT_MQ_URL')],
        exchange: this.configService.get('RABBIT_MQ_EXCHANGE'),
        exchangeType: 'topic',
        queue: this.configService.get('RABBIT_MQ_QUEUE'),
        queueOptions: {
          durable: false,
          ...(options.queueOptions ?? {})
        },
        wildcards: true,
        ...options
      }
    };
  }
}