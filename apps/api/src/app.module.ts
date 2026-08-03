import { Module } from "@nestjs/common";
import { PathfinderModule } from "./pathfinder/pathfinder.module";

@Module({
  imports: [PathfinderModule]
})
export class AppModule {}
