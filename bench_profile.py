import cProfile
import bench
from pstats import SortKey

ps = cProfile.Profile()
ps.enable()
bench.setup(maxpaths=2**20)
bench.unsetup()
ps.disable()
ps.create_stats()
ps.print_stats(SortKey.CUMULATIVE)