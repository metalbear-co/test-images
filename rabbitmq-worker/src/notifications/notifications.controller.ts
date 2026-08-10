import { Controller, Logger } from '@nestjs/common';
import { EventPattern, Payload, Ctx, RmqContext } from '@nestjs/microservices';

@Controller()
export class NotificationsController {
  private readonly logger = new Logger(NotificationsController.name);

  @EventPattern('order.created')
  async handleCreated(
    @Payload() data: any,
    @Ctx() context: RmqContext,
  ) {
    this.logger.log(`Send email for order: ${data.id}`);
  }

  @EventPattern('order.cancelled')
  handleCancelled(@Payload() data: any) {
    this.logger.log(`Send cancellation email: ${data.id}`);
  }
}