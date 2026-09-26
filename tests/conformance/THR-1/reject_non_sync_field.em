#$ test: compile-fail
#$ rules: THR-1, THR-8, THR-9
#$ profiles: debug
#$ error[E7001]: field `Shared.item` has type `Local`, which is not Send or Sync

class Local:
    value: int = 0

@sync
class Shared:
    item: Local

    fn init(mut self, item: Local):
        self.item = item
