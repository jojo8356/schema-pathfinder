import { Module } from "@nestjs/common";
import { PathfinderController } from "./pathfinder.controller";
import { PathfinderService } from "./pathfinder.service";

@Module({
  controllers: [PathfinderController],
  providers: [PathfinderService],
  exports: [PathfinderService]
})
export class PathfinderModule {}
