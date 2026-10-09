"""The annotation sweep removes its outputs on success and on an exception."""
import contextlib
import importlib.util
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'tasks/impl-0.9.9/annotations.py'


class AnnotationCleanup(unittest.TestCase):
    def check_cleanup(self, fail):
        spec = importlib.util.spec_from_file_location('annotation_cleanup_subject', SOURCE)
        runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(runner)
        outputs = []

        def check(path):
            output = Path(runner.out_dir(path))
            output.mkdir(parents=True)
            (output / 'program').write_bytes(b'compiled test output')
            outputs.append(output)
            if fail:
                raise RuntimeError('simulated annotation failure')
            return []

        with tempfile.TemporaryDirectory(prefix='annotation-fixture-') as fixture:
            (Path(fixture) / 'accept_cleanup.em').write_text('fn main():\n    pass\n', encoding='utf-8')
            with patch.object(runner.sys, 'argv', [str(SOURCE), fixture]), \
                    patch.object(runner, 'check', check), \
                    contextlib.redirect_stdout(io.StringIO()):
                if fail:
                    with self.assertRaisesRegex(RuntimeError, 'simulated annotation failure'):
                        runner.main()
                else:
                    runner.main()
        self.assertEqual(len(outputs), 1)
        self.assertFalse(outputs[0].exists())
        self.assertFalse(outputs[0].parent.exists())

    def test_success_removes_output(self):
        self.check_cleanup(False)

    def test_exception_removes_output(self):
        self.check_cleanup(True)


if __name__ == '__main__':
    unittest.main()
