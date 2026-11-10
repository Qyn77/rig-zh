import sys
from pathlib import Path
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from accept_deepseek import ENGLISH, validate_result

class AcceptanceTests(unittest.TestCase):
    def sample(self):
        return {'markdown': ENGLISH.replace('Providers and conversation memory', '服务商与对话记忆'),
                'rust': 'use rig::{catalog::Catalog, memory::MemoryStore, AgentBuilder};\nfn main() {}'}

    def test_comments_may_explain_which_obsolete_apis_were_replaced(self):
        sample = self.sample()
        sample['rust'] += '\n// Replaces ProviderRef and InMemoryConversationMemory\n'
        validate_result(sample)

    def test_real_obsolete_import_is_rejected(self):
        sample = self.sample()
        sample['rust'] += '\nuse rig::providers::registry::ProviderRef;\n'
        with self.assertRaisesRegex(ValueError, 'obsolete API'):
            validate_result(sample)

    def test_type_names_in_comments_do_not_prove_migration(self):
        sample = self.sample()
        sample['rust'] = '// Catalog MemoryStore AgentBuilder\nfn main() {}'
        with self.assertRaisesRegex(ValueError, 'omitted required'):
            validate_result(sample)
