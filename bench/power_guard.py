"""Whether the machine is on mains power (Windows' GetSystemPowerStatus): timings on battery are
invalid (owner, 2026-10-02)."""
import ctypes


class _Status(ctypes.Structure):
    _fields_ = [('ACLineStatus', ctypes.c_ubyte), ('BatteryFlag', ctypes.c_ubyte), ('BatteryLifePercent', ctypes.c_ubyte),
                ('SystemStatusFlag', ctypes.c_ubyte), ('BatteryLifeTime', ctypes.c_ulong), ('BatteryFullLifeTime', ctypes.c_ulong)]


def on_mains():
    status = _Status()
    ctypes.windll.kernel32.GetSystemPowerStatus(ctypes.byref(status))
    return status.ACLineStatus == 1


def require_mains(where):
    if not on_mains():
        raise SystemExit(f'ON BATTERY at {where}: stopping; timings on battery are invalid')
