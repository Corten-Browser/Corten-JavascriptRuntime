# Claude Code Orchestration System - Implementation Code

## 1. Context Window Manager (orchestration/context_manager.py)

```python
from pathlib import Path
from typing import Dict, Tuple
import json
from datetime import datetime

class ContextWindowManager:
    """Ensure components never exceed Claude's context window"""
    
    # Claude's context window limit
    MAX_CONTEXT_TOKENS = 200_000
    
    # Token overhead breakdown (updated for new 200k context limits)
    OVERHEAD_TOKENS = {
        'claude_md': 1_500,
        'work_instructions': 500,
        'system_prompts': 2_000,
        'contracts': 2_000,
        'response_buffer': 15_000,
        'total': 21_000
    }
    
    def calculate_component_limits(self, task_type: str = 'simple_edit') -> Dict:
        """Calculate safe component size limits"""
        
        safety_margin = self.SAFETY_MARGINS.get(task_type, 0.5)
        available_tokens = self.MAX_CONTEXT_TOKENS * safety_margin
        
        # Token budget breakdown (updated for 200k context):
        # - CLAUDE.md instructions: ~1,500 tokens
        # - Work instructions: ~500 tokens
        # - System prompts: ~2,000 tokens
        # - Contracts: ~2,000 tokens
        # - Response buffer: ~15,000 tokens
        # Total overhead: 21,000 tokens (reduced from 28,000)

        overhead_tokens = 21_000
        source_code_budget = available_tokens - overhead_tokens
        
        # Approximation: 1 line of code ≈ 10 tokens average
        # (varies by language and style)
        max_lines = int(source_code_budget / 10)
        
        return {
            'max_total_tokens': available_tokens,
            'source_code_tokens': source_code_budget,
            'max_lines': max_lines,
            'recommended_lines': int(max_lines * 0.8),  # 80% for safety
            'absolute_max_lines': int(max_lines * 1.2)  # Never exceed
        }
    
    def analyze_component(self, component_path: Path) -> Dict:
        """Analyze if a component fits within context limits"""
        
        total_tokens = 0
        line_count = 0
        file_details = []
        
        # Count source code
        for file in component_path.glob("**/*"):
            if file.is_file() and file.suffix in ['.py', '.rs', '.js', '.ts', '.tsx']:
                try:
                    content = file.read_text()
                    lines = len(content.splitlines())
                    line_count += lines
                    # Rough estimate: ~10 tokens per line
                    file_tokens = lines * 10
                    total_tokens += file_tokens
                    
                    file_details.append({
                        'path': str(file.relative_to(component_path)),
                        'lines': lines,
                        'estimated_tokens': file_tokens
                    })
                except:
                    pass
        
        # Add overhead (updated for 200k context)
        total_tokens += 21_000  # Standard overhead
        
        # Calculate status
        if total_tokens < self.MAX_CONTEXT_TOKENS * 0.5:
            status = 'green'
            message = 'Well within limits'
        elif total_tokens < self.MAX_CONTEXT_TOKENS * 0.7:
            status = 'yellow' 
            message = 'Approaching limits - consider splitting'
        elif total_tokens < self.MAX_CONTEXT_TOKENS * 0.9:
            status = 'orange'
            message = 'Near limits - split recommended'
        else:
            status = 'red'
            message = 'EXCEEDS LIMITS - split required immediately'
        
        return {
            'component_name': component_path.name,
            'line_count': line_count,
            'estimated_tokens': total_tokens,
            'percentage_used': (total_tokens / self.MAX_CONTEXT_TOKENS) * 100,
            'status': status,
            'message': message,
            'recommended_action': self._get_split_recommendation(line_count, total_tokens),
            'file_details': file_details,
            'timestamp': datetime.now().isoformat()
        }
    
    def _get_split_recommendation(self, lines: int, tokens: int) -> str:
        """Recommend splitting strategy (updated for new 200k context limits)"""

        if tokens < 120_000:
            return "No action needed - optimal size"
        elif tokens < 150_000:
            return "Monitor growth, prepare split strategy if growth continues"
        elif tokens < 170_000:
            return "Plan split soon - recommend 2-3 components based on natural boundaries"
        elif tokens < 180_000:
            return "SPLIT IMMEDIATELY - component exceeds safe limits"
        else:
            return "URGENT: EMERGENCY SPLIT REQUIRED - split into 3+ components immediately"
    
    def save_analysis(self, component_path: Path, output_dir: Path = None):
        """Save analysis to JSON file"""
        
        analysis = self.analyze_component(component_path)
        
        if output_dir is None:
            output_dir = Path("orchestration/token-tracking")
        
        output_dir.mkdir(parents=True, exist_ok=True)
        
        output_file = output_dir / f"{component_path.name}_analysis.json"
        with open(output_file, 'w') as f:
            json.dump(analysis, f, indent=2)
        
        return output_file


class TokenTracker:
    """Track token usage across all components"""
    
    def __init__(self):
        self.tracker_file = Path("orchestration/token-tracker.json")
        self.context_manager = ContextWindowManager()
    
    def update_all_components(self):
        """Update token tracking for all components"""
        
        components_dir = Path("components")
        tracking_data = {
            'timestamp': datetime.now().isoformat(),
            'components': {},
            'summary': {
                'total_components': 0,
                'components_near_limit': 0,
                'components_over_limit': 0
            }
        }
        
        for component_path in components_dir.iterdir():
            if component_path.is_dir() and not component_path.name.startswith('_'):
                analysis = self.context_manager.analyze_component(component_path)
                
                tracking_data['components'][component_path.name] = {
                    'tokens': analysis['estimated_tokens'],
                    'lines': analysis['line_count'],
                    'status': analysis['status'],
                    'percentage': analysis['percentage_used']
                }
                
                tracking_data['summary']['total_components'] += 1
                
                if analysis['status'] in ['orange', 'red']:
                    tracking_data['summary']['components_near_limit'] += 1
                
                if analysis['status'] == 'red':
                    tracking_data['summary']['components_over_limit'] += 1
        
        # Save tracking data
        self.tracker_file.parent.mkdir(parents=True, exist_ok=True)
        with open(self.tracker_file, 'w') as f:
            json.dump(tracking_data, f, indent=2)
        
        return tracking_data
    
    def get_critical_components(self):
        """Get list of components that need immediate attention"""
        
        if not self.tracker_file.exists():
            self.update_all_components()
        
        with open(self.tracker_file, 'r') as f:
            data = json.load(f)
        
        critical = []
        for name, info in data['components'].items():
            if info['status'] in ['orange', 'red']:
                critical.append({
                    'name': name,
                    'status': info['status'],
                    'tokens': info['tokens'],
                    'action_required': 'SPLIT IMMEDIATELY' if info['status'] == 'red' else 'Plan split soon'
                })
        
        return critical
```

## 2. Claude Code Analyzer (orchestration/claude_code_analyzer.py)

```python
import subprocess
import json
import tempfile
from pathlib import Path
from typing import Dict, List, Optional
from datetime import datetime

class ClaudeCodeAnalyzer:
    """Use Claude Code sub-agents for analysis tasks"""
    
    def __init__(self):
        self.analysis_workspace = Path("orchestration/analysis_workspace")
        self.analysis_workspace.mkdir(parents=True, exist_ok=True)
    
    def analyze_with_claude_code(self, task: str, target_path: Path, 
                                 additional_context: Dict = None) -> Dict:
        """Launch Claude Code sub-agent for analysis"""
        
        # Create temporary analysis directory for this task
        task_id = f"{task}_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
        task_dir = self.analysis_workspace / task_id
        task_dir.mkdir(exist_ok=True)
        
        # Create specialized CLAUDE.md for analysis task
        analysis_instructions = self._create_analysis_instructions(task, target_path, additional_context)
        (task_dir / "CLAUDE.md").write_text(analysis_instructions)
        
        # Create analysis script that Claude Code will execute
        analysis_script = self._create_analysis_script(task, target_path, additional_context)
        (task_dir / "analyze.py").write_text(analysis_script)
        
        # Create task instruction file
        task_instruction = f"""Execute the following analysis task:
        
1. Read and understand the analyze.py script
2. Run the script to analyze {target_path}
3. Review the generated analysis_output.json
4. Enhance the analysis with your insights
5. Save final results to analysis_output.json

Task: {task}
Target: {target_path}
"""
        (task_dir / "task.md").write_text(task_instruction)
        
        # Launch Claude Code in analysis directory
        print(f"Launching Claude Code sub-agent for task: {task}")
        result = subprocess.run(
            ["claude", "code", "--task", "Complete the analysis task described in task.md"],
            cwd=task_dir,
            capture_output=True,
            text=True,
            timeout=300  # 5 minute timeout
        )
        
        # Read results
        output_file = task_dir / "analysis_output.json"
        if output_file.exists():
            results = json.loads(output_file.read_text())
            results['task_id'] = task_id
            results['completed'] = True
            return results
        else:
            return {
                "task_id": task_id,
                "completed": False,
                "error": "Analysis failed",
                "stdout": result.stdout,
                "stderr": result.stderr
            }
    
    def _create_analysis_instructions(self, task: str, target: Path, 
                                     context: Dict = None) -> str:
        """Create CLAUDE.md for analysis sub-agent"""
        
        base_instructions = f"""# Analysis Sub-Agent: {task}

You are a specialized analysis agent with a single focused task.

## Your Task
{self._get_task_description(task)}

## Target Path
{target}

## Process
1. Execute the provided analyze.py script
2. Review and enhance the analysis results
3. Save comprehensive findings to analysis_output.json

## Output Requirements
- Valid JSON format only
- Structured, actionable recommendations
- Clear identification of issues and opportunities
"""
        
        if context:
            base_instructions += f"\n## Additional Context\n{json.dumps(context, indent=2)}"
        
        return base_instructions
    
    def _get_task_description(self, task: str) -> str:
        """Get detailed task description"""
        
        descriptions = {
            "identify_components": """
Analyze the codebase to identify natural component boundaries.
Look for:
- Logical separation of concerns
- API boundaries
- Data model groupings
- Feature modules
- Shared utilities
Output component structure with size estimates.""",
            
            "plan_split": """
Plan how to split an oversized component.
Identify:
- Natural split points
- Dependencies that need refactoring
- Shared code to extract
- New component names and responsibilities
Output detailed splitting plan.""",
            
            "migration_analysis": """
Analyze existing project for migration to orchestrated architecture.
Identify:
- Current structure and patterns
- Component candidates
- Migration challenges
- Effort estimates
Output comprehensive migration plan.""",
            
            "dependency_mapping": """
Map dependencies between components.
Identify:
- Import relationships
- API calls
- Shared data structures
- Circular dependencies
Output dependency graph and issues."""
        }
        
        return descriptions.get(task, f"Analyze the codebase for {task}")
    
    def _create_analysis_script(self, task: str, target: Path, 
                               context: Dict = None) -> str:
        """Create Python script for Claude Code to execute"""
        
        if task == "identify_components":
            return self._create_component_identification_script(target)
        elif task == "plan_split":
            return self._create_split_planning_script(target)
        elif task == "migration_analysis":
            return self._create_migration_analysis_script(target)
        elif task == "dependency_mapping":
            return self._create_dependency_mapping_script(target)
        else:
            return self._create_generic_analysis_script(task, target)
    
    def _create_component_identification_script(self, target: Path) -> str:
        """Create script for component identification"""
        
        return f"""#!/usr/bin/env python3
\"\"\"Component identification script for Claude Code execution\"\"\"

import json
import os
from pathlib import Path
from collections import defaultdict

def analyze_components(root_path):
    \"\"\"Analyze codebase and identify components\"\"\"
    
    root = Path(root_path)
    components = []
    
    # Analyze directory structure
    for subdir in root.iterdir():
        if subdir.is_dir() and not subdir.name.startswith('.'):
            component = analyze_directory(subdir)
            if component['estimated_tokens'] > 0:
                components.append(component)
    
    # If flat structure, analyze by file patterns
    if len(components) == 0:
        components = analyze_by_patterns(root)
    
    # Generate recommendations
    recommendations = generate_recommendations(components)
    
    return {{
        'components': components,
        'recommendations': recommendations,
        'total_components': len(components),
        'total_tokens': sum(c['estimated_tokens'] for c in components)
    }}

def analyze_directory(directory):
    \"\"\"Analyze a single directory\"\"\"
    
    component = {{
        'name': directory.name,
        'path': str(directory),
        'files': [],
        'estimated_tokens': 0,
        'line_count': 0,
        'primary_language': None,
        'purpose': infer_purpose(directory)
    }}
    
    language_counts = defaultdict(int)
    
    for file in directory.rglob('*'):
        if file.is_file() and is_source_file(file):
            try:
                lines = len(file.read_text().splitlines())
                component['files'].append(str(file.relative_to(directory)))
                component['line_count'] += lines
                component['estimated_tokens'] += lines * 10
                
                # Track language
                language_counts[file.suffix] += lines
            except:
                pass
    
    # Determine primary language
    if language_counts:
        component['primary_language'] = max(language_counts.items(), key=lambda x: x[1])[0]
    
    return component

def is_source_file(file):
    \"\"\"Check if file is source code\"\"\"
    source_extensions = {{'.py', '.rs', '.js', '.ts', '.tsx', '.jsx', '.java', '.cpp', '.c', '.h', '.go'}}
    return file.suffix in source_extensions

def infer_purpose(directory):
    \"\"\"Infer component purpose from directory name and contents\"\"\"
    
    name = directory.name.lower()
    
    # Check directory name patterns
    patterns = {{
        'api': ['api', 'endpoint', 'route', 'controller'],
        'data': ['model', 'schema', 'database', 'entity'],
        'ui': ['ui', 'frontend', 'component', 'view', 'page'],
        'service': ['service', 'business', 'logic', 'core'],
        'utils': ['util', 'helper', 'common', 'shared'],
        'test': ['test', 'spec', 'fixture'],
    }}
    
    for purpose, keywords in patterns.items():
        if any(keyword in name for keyword in keywords):
            return purpose
    
    return 'general'

def analyze_by_patterns(root):
    \"\"\"Analyze flat structure by file patterns\"\"\"
    
    # Group files by detected patterns
    api_files = []
    ui_files = []
    data_files = []
    service_files = []
    test_files = []
    util_files = []
    
    for file in root.rglob('*'):
        if file.is_file() and is_source_file(file):
            content_lower = file.read_text().lower()
            name_lower = file.name.lower()
            
            if 'route' in content_lower or 'endpoint' in name_lower:
                api_files.append(file)
            elif 'component' in content_lower or 'render' in content_lower:
                ui_files.append(file)
            elif 'model' in name_lower or 'schema' in name_lower:
                data_files.append(file)
            elif 'service' in name_lower or 'business' in name_lower:
                service_files.append(file)
            elif 'test' in name_lower or 'spec' in name_lower:
                test_files.append(file)
            else:
                util_files.append(file)
    
    components = []
    
    # Create components from file groups
    file_groups = {{
        'api-layer': api_files,
        'ui-layer': ui_files,
        'data-layer': data_files,
        'service-layer': service_files,
        'test-suite': test_files,
        'utilities': util_files
    }}
    
    for name, files in file_groups.items():
        if files:
            components.append(create_component_from_files(name, files))
    
    return components

def create_component_from_files(name, files):
    \"\"\"Create component info from file list\"\"\"
    
    total_lines = 0
    file_list = []
    
    for file in files:
        try:
            lines = len(file.read_text().splitlines())
            total_lines += lines
            file_list.append(str(file))
        except:
            pass
    
    return {{
        'name': name,
        'files': file_list,
        'line_count': total_lines,
        'estimated_tokens': total_lines * 10,
        'purpose': name.split('-')[0]
    }}

def generate_recommendations(components):
    \"\"\"Generate recommendations based on analysis\"\"\"
    
    recommendations = []
    
    for comp in components:
        tokens = comp['estimated_tokens']

        if tokens > 170000:
            recommendations.append({{
                'component': comp['name'],
                'action': 'SPLIT IMMEDIATELY',
                'reason': f'Exceeds safe limits with {{tokens:,}} tokens',
                'priority': 'critical'
            }})
        elif tokens > 150000:
            recommendations.append({{
                'component': comp['name'],
                'action': 'Plan split soon',
                'reason': f'Near hard limits with {{tokens:,}} tokens',
                'priority': 'high'
            }})
        elif tokens > 120000:
            recommendations.append({{
                'component': comp['name'],
                'action': 'Monitor growth',
                'reason': f'Approaching recommended size with {{tokens:,}} tokens',
                'priority': 'medium'
            }})
    
    return recommendations

# Execute analysis
results = analyze_components('{target}')

# Save results
with open('analysis_output.json', 'w') as f:
    json.dump(results, f, indent=2)

print(f"Analysis complete. Found {{results['total_components']}} components.")
print(f"Total estimated tokens: {{results['total_tokens']:,}}")
"""
    
    def _create_split_planning_script(self, target: Path) -> str:
        """Create script for planning component splits"""
        
        return f"""#!/usr/bin/env python3
\"\"\"Component split planning script\"\"\"

import json
from pathlib import Path
from collections import defaultdict

def plan_component_split(component_path):
    \"\"\"Plan how to split an oversized component\"\"\"
    
    path = Path(component_path)
    
    # Analyze current structure
    analysis = analyze_component_structure(path)
    
    # Determine split strategy
    if analysis['has_layers']:
        strategy = 'horizontal'  # Split by architectural layers
    elif analysis['has_features']:
        strategy = 'vertical'    # Split by features
    else:
        strategy = 'hybrid'       # Combination
    
    # Generate split plan
    plan = generate_split_plan(analysis, strategy)
    
    return {{
        'component': path.name,
        'current_size': analysis['total_tokens'],
        'strategy': strategy,
        'new_components': plan['components'],
        'shared_code': plan['shared'],
        'migration_steps': plan['steps']
    }}

def analyze_component_structure(path):
    \"\"\"Analyze component internal structure\"\"\"
    
    structure = {{
        'total_tokens': 0,
        'has_layers': False,
        'has_features': False,
        'directories': [],
        'file_groups': defaultdict(list)
    }}
    
    # Check for layer-based organization
    layer_dirs = ['controllers', 'services', 'models', 'repositories']
    for layer_dir in layer_dirs:
        if (path / layer_dir).exists():
            structure['has_layers'] = True
            break
    
    # Check for feature-based organization
    for subdir in path.iterdir():
        if subdir.is_dir() and not subdir.name.startswith('.'):
            structure['directories'].append(subdir.name)
            if not any(layer in subdir.name.lower() for layer in layer_dirs):
                structure['has_features'] = True
    
    # Group files by type
    for file in path.rglob('*'):
        if file.is_file() and file.suffix in ['.py', '.js', '.ts', '.rs']:
            file_type = categorize_file(file)
            structure['file_groups'][file_type].append(str(file.relative_to(path)))
            
            # Count tokens
            try:
                lines = len(file.read_text().splitlines())
                structure['total_tokens'] += lines * 10
            except:
                pass
    
    return structure

def categorize_file(file):
    \"\"\"Categorize file by its likely purpose\"\"\"
    
    name = file.name.lower()
    
    if 'controller' in name or 'route' in name or 'endpoint' in name:
        return 'api'
    elif 'service' in name or 'business' in name:
        return 'service'
    elif 'model' in name or 'entity' in name or 'schema' in name:
        return 'data'
    elif 'repository' in name or 'dao' in name:
        return 'repository'
    elif 'util' in name or 'helper' in name:
        return 'utility'
    elif 'test' in name or 'spec' in name:
        return 'test'
    else:
        return 'other'

def generate_split_plan(analysis, strategy):
    \"\"\"Generate specific split plan based on strategy\"\"\"
    
    plan = {{
        'components': [],
        'shared': [],
        'steps': []
    }}
    
    if strategy == 'horizontal':
        # Split by layers
        plan['components'] = [
            {{
                'name': 'api-layer',
                'files': analysis['file_groups']['api'],
                'estimated_tokens': len(analysis['file_groups']['api']) * 200
            }},
            {{
                'name': 'business-logic',
                'files': analysis['file_groups']['service'],
                'estimated_tokens': len(analysis['file_groups']['service']) * 300
            }},
            {{
                'name': 'data-layer',
                'files': analysis['file_groups']['data'] + analysis['file_groups']['repository'],
                'estimated_tokens': (len(analysis['file_groups']['data']) + 
                                   len(analysis['file_groups']['repository'])) * 150
            }}
        ]
        
    elif strategy == 'vertical':
        # Split by features
        # This is simplified - real implementation would analyze feature boundaries
        num_features = max(2, min(4, len(analysis['directories'])))
        for i in range(num_features):
            plan['components'].append({{
                'name': f'feature-{{i+1}}',
                'files': [],  # Would be populated by feature analysis
                'estimated_tokens': analysis['total_tokens'] // num_features
            }})
    
    else:
        # Hybrid strategy
        plan['components'] = [
            {{'name': 'core-services', 'estimated_tokens': analysis['total_tokens'] // 3}},
            {{'name': 'api-gateway', 'estimated_tokens': analysis['total_tokens'] // 3}},
            {{'name': 'shared-utils', 'estimated_tokens': analysis['total_tokens'] // 3}}
        ]
    
    # Identify shared code
    plan['shared'] = analysis['file_groups']['utility']
    
    # Generate migration steps
    plan['steps'] = [
        'Create new component directories',
        'Move files to appropriate components',
        'Update import statements',
        'Extract shared utilities',
        'Create API contracts between components',
        'Update component CLAUDE.md files',
        'Test each component independently',
        'Validate integration'
    ]
    
    return plan

# Execute planning
result = plan_component_split('{target}')

# Save plan
with open('analysis_output.json', 'w') as f:
    json.dump(result, f, indent=2)

print(f"Split plan created for {{result['component']}}")
print(f"Strategy: {{result['strategy']}}")
print(f"New components: {{len(result['new_components'])}}")
"""
    
    def _create_migration_analysis_script(self, target: Path) -> str:
        """Create script for migration analysis"""
        
        return f"""#!/usr/bin/env python3
\"\"\"Project migration analysis script\"\"\"

import json
from pathlib import Path
from typing import Dict, List

def analyze_for_migration(project_path):
    \"\"\"Analyze existing project for migration to orchestrated architecture\"\"\"
    
    path = Path(project_path)
    
    # Detect project type
    project_type = detect_project_type(path)
    
    # Find component boundaries
    components = identify_components(path)
    
    # Analyze complexity
    complexity = assess_complexity(path)
    
    # Generate migration plan
    plan = create_migration_plan(components, complexity)
    
    return {{
        'project_path': str(path),
        'project_type': project_type,
        'total_lines': complexity['total_lines'],
        'components': components,
        'complexity': complexity,
        'migration_plan': plan,
        'estimated_effort': estimate_effort(complexity)
    }}

def detect_project_type(path):
    \"\"\"Detect the type of project\"\"\"
    
    # Check for framework indicators
    if (path / 'package.json').exists():
        return 'javascript/node'
    elif (path / 'Cargo.toml').exists():
        return 'rust'
    elif (path / 'requirements.txt').exists() or (path / 'setup.py').exists():
        return 'python'
    elif (path / 'go.mod').exists():
        return 'golang'
    else:
        return 'unknown'

def identify_components(path):
    \"\"\"Identify natural component boundaries\"\"\"
    
    components = []
    
    # Check existing structure
    for subdir in path.iterdir():
        if subdir.is_dir() and not subdir.name.startswith('.'):
            if is_component_candidate(subdir):
                components.append(analyze_component_candidate(subdir))
    
    # If no clear structure, analyze by patterns
    if not components:
        components = analyze_by_code_patterns(path)
    
    return components

def is_component_candidate(directory):
    \"\"\"Check if directory could be a component\"\"\"
    
    # Has source files
    has_code = any(
        f.suffix in ['.py', '.js', '.ts', '.rs', '.go', '.java']
        for f in directory.rglob('*') if f.is_file()
    )
    
    # Not a build/dependency directory
    not_excluded = directory.name not in [
        'node_modules', 'venv', '.venv', 'target', 
        'dist', 'build', '__pycache__'
    ]
    
    return has_code and not_excluded

def analyze_component_candidate(directory):
    \"\"\"Analyze a potential component\"\"\"
    
    total_lines = 0
    file_count = 0
    
    for file in directory.rglob('*'):
        if file.is_file() and file.suffix in ['.py', '.js', '.ts', '.rs', '.go']:
            try:
                lines = len(file.read_text().splitlines())
                total_lines += lines
                file_count += 1
            except:
                pass
    
    return {{
        'name': directory.name,
        'path': str(directory),
        'lines': total_lines,
        'files': file_count,
        'estimated_tokens': total_lines * 10,
        'type': infer_component_type(directory)
    }}

def infer_component_type(directory):
    \"\"\"Infer component type from contents\"\"\"
    
    name = directory.name.lower()
    
    if 'api' in name or 'server' in name:
        return 'backend'
    elif 'ui' in name or 'frontend' in name or 'client' in name:
        return 'frontend'
    elif 'lib' in name or 'common' in name:
        return 'library'
    elif 'test' in name:
        return 'test'
    else:
        return 'module'

def analyze_by_code_patterns(path):
    \"\"\"Analyze code patterns when no clear structure exists\"\"\"
    
    # Simplified pattern analysis
    # Real implementation would do deeper code analysis
    
    return [
        {{
            'name': 'monolith',
            'path': str(path),
            'lines': count_total_lines(path),
            'type': 'monolith',
            'estimated_tokens': count_total_lines(path) * 10
        }}
    ]

def count_total_lines(path):
    \"\"\"Count total lines in project\"\"\"
    
    total = 0
    for file in path.rglob('*'):
        if file.is_file() and file.suffix in ['.py', '.js', '.ts', '.rs', '.go']:
            try:
                total += len(file.read_text().splitlines())
            except:
                pass
    return total

def assess_complexity(path):
    \"\"\"Assess project complexity\"\"\"
    
    return {{
        'total_lines': count_total_lines(path),
        'file_count': len(list(path.rglob('*.*'))),
        'directory_depth': max(len(p.parts) for p in path.rglob('*') if p.is_file()),
        'has_tests': any('test' in str(p).lower() for p in path.rglob('*')),
        'has_ci': (path / '.github/workflows').exists() or (path / '.gitlab-ci.yml').exists()
    }}

def create_migration_plan(components, complexity):
    \"\"\"Create migration plan\"\"\"
    
    phases = []
    
    # Phase 1: Preparation
    phases.append({{
        'phase': 1,
        'name': 'Preparation',
        'tasks': [
            'Back up existing code',
            'Set up orchestration structure',
            'Install orchestration tools',
            'Create initial contracts'
        ]
    }})
    
    # Phase 2: Component extraction
    for component in components:
        phases.append({{
            'phase': 2,
            'name': f'Extract {{component["name"]}}',
            'tasks': [
                f'Create component directory for {{component["name"]}}',
                f'Move files to component directory',
                f'Create CLAUDE.md for component',
                f'Set up local git repository',
                f'Create component tests'
            ]
        }})
    
    # Phase 3: Integration
    phases.append({{
        'phase': 3,
        'name': 'Integration',
        'tasks': [
            'Define API contracts',
            'Set up integration tests',
            'Validate component communication',
            'Performance testing'
        ]
    }})
    
    return phases

def estimate_effort(complexity):
    \"\"\"Estimate migration effort\"\"\"
    
    lines = complexity['total_lines']
    
    if lines < 1000:
        return 'Low (2-4 hours)'
    elif lines < 5000:
        return 'Medium (1-2 days)'
    elif lines < 20000:
        return 'High (3-5 days)'
    elif lines < 100000:
        return 'Very High (1-2 weeks)'
    else:
        return 'Extensive (2+ weeks)'

# Execute analysis
result = analyze_for_migration('{target}')

# Save results
with open('analysis_output.json', 'w') as f:
    json.dump(result, f, indent=2)

print(f"Migration analysis complete for {{result['project_type']}} project")
print(f"Found {{len(result['components'])}} components")
print(f"Estimated effort: {{result['estimated_effort']}}")
"""
    
    def _create_dependency_mapping_script(self, target: Path) -> str:
        """Create script for dependency mapping"""
        
        return f"""#!/usr/bin/env python3
\"\"\"Dependency mapping script\"\"\"

import json
import ast
import re
from pathlib import Path
from collections import defaultdict

def map_dependencies(component_path):
    \"\"\"Map all dependencies in a component\"\"\"
    
    path = Path(component_path)
    
    # Analyze imports
    imports = analyze_imports(path)
    
    # Build dependency graph
    graph = build_dependency_graph(imports)
    
    # Detect issues
    issues = detect_dependency_issues(graph)
    
    return {{
        'component': path.name,
        'dependencies': graph,
        'issues': issues,
        'metrics': calculate_metrics(graph)
    }}

def analyze_imports(path):
    \"\"\"Analyze all imports in component\"\"\"
    
    imports = defaultdict(set)
    
    for file in path.rglob('*.py'):
        try:
            tree = ast.parse(file.read_text())
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        imports[str(file)].add(alias.name)
                elif isinstance(node, ast.ImportFrom):
                    if node.module:
                        imports[str(file)].add(node.module)
        except:
            pass
    
    # Also check JavaScript/TypeScript files
    for file in path.rglob('*.js'):
        try:
            content = file.read_text()
            # Simple regex for imports
            import_pattern = r'import.*from\\s+[\'"]([^\'"]+)[\'"]'
            for match in re.findall(import_pattern, content):
                imports[str(file)].add(match)
        except:
            pass
    
    return imports

def build_dependency_graph(imports):
    \"\"\"Build dependency graph from imports\"\"\"
    
    graph = {{
        'internal': defaultdict(set),
        'external': defaultdict(set)
    }}
    
    for file, deps in imports.items():
        for dep in deps:
            if dep.startswith('.'):
                # Relative import - internal
                graph['internal'][file].add(dep)
            else:
                # External dependency
                graph['external'][file].add(dep)
    
    return graph

def detect_dependency_issues(graph):
    \"\"\"Detect dependency issues\"\"\"
    
    issues = []
    
    # Check for circular dependencies
    # Simplified check - real implementation would be more thorough
    for file1, deps1 in graph['internal'].items():
        for file2, deps2 in graph['internal'].items():
            if file1 != file2:
                if file2 in deps1 and file1 in deps2:
                    issues.append({{
                        'type': 'circular',
                        'files': [file1, file2],
                        'severity': 'high'
                    }})
    
    # Check for too many dependencies
    for file, deps in graph['external'].items():
        if len(deps) > 20:
            issues.append({{
                'type': 'excessive_dependencies',
                'file': file,
                'count': len(deps),
                'severity': 'medium'
            }})
    
    return issues

def calculate_metrics(graph):
    \"\"\"Calculate dependency metrics\"\"\"
    
    return {{
        'total_files': len(set(graph['internal'].keys()) | set(graph['external'].keys())),
        'internal_dependencies': sum(len(deps) for deps in graph['internal'].values()),
        'external_dependencies': sum(len(deps) for deps in graph['external'].values()),
        'avg_dependencies_per_file': (
            sum(len(deps) for deps in graph['internal'].values()) + 
            sum(len(deps) for deps in graph['external'].values())
        ) / max(1, len(set(graph['internal'].keys()) | set(graph['external'].keys())))
    }}

# Execute dependency mapping
result = map_dependencies('{target}')

# Save results
with open('analysis_output.json', 'w') as f:
    json.dump(result, f, indent=2)

print(f"Dependency mapping complete for {{result['component']}}")
print(f"Found {{len(result['issues'])}} issues")
"""
    
    def _create_generic_analysis_script(self, task: str, target: Path) -> str:
        """Create generic analysis script"""
        
        return f"""#!/usr/bin/env python3
\"\"\"Generic analysis script for {task}\"\"\"

import json
from pathlib import Path

def analyze(target_path):
    \"\"\"Perform generic analysis\"\"\"
    
    path = Path(target_path)
    
    return {{
        'task': '{task}',
        'target': str(path),
        'exists': path.exists(),
        'is_directory': path.is_dir() if path.exists() else None,
        'analysis': 'Generic analysis completed'
    }}

# Execute analysis
result = analyze('{target}')

# Save results
with open('analysis_output.json', 'w') as f:
    json.dump(result, f, indent=2)

print(f"Analysis complete for {task}")
"""
```

## 3. Component Splitter (orchestration/component_splitter.py)

```python
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import json
import shutil
from datetime import datetime
from claude_code_analyzer import ClaudeCodeAnalyzer
from context_manager import ContextWindowManager

class AdaptiveComponentSplitter:
    """Split components based on token limits"""
    
    def __init__(self):
        self.context_manager = ContextWindowManager()
        self.analyzer = ClaudeCodeAnalyzer()
        self.split_history_file = Path("orchestration/split-history.json")
    
    def check_and_split(self, component_path: Path) -> Dict:
        """Check component size and split if necessary"""
        
        # Analyze component
        analysis = self.context_manager.analyze_component(component_path)
        
        if analysis['status'] in ['orange', 'red']:
            # Needs splitting
            print(f"Component {component_path.name} needs splitting: {analysis['message']}")
            return self.execute_split(component_path, analysis)
        else:
            return {
                'action': 'none',
                'reason': 'Component within safe limits',
                'analysis': analysis
            }
    
    def execute_split(self, component_path: Path, analysis: Dict) -> Dict:
        """Execute component split"""
        
        print(f"Planning split for {component_path.name}...")
        
        # Use Claude Code to plan the split
        split_plan = self.analyzer.analyze_with_claude_code(
            task="plan_split",
            target_path=component_path,
            additional_context={'current_analysis': analysis}
        )
        
        if not split_plan.get('completed'):
            return {
                'action': 'failed',
                'reason': 'Split planning failed',
                'error': split_plan.get('error')
            }
        
        # Execute the split plan
        result = self._execute_split_plan(component_path, split_plan)
        
        # Record in history
        self._record_split(component_path, split_plan, result)
        
        return result
    
    def _execute_split_plan(self, component_path: Path, plan: Dict) -> Dict:
        """Execute the split plan"""
        
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        
        # Create backup
        backup_path = component_path.parent / f"_{component_path.name}_backup_{timestamp}"
        shutil.copytree(component_path, backup_path)
        
        # Create new component directories
        new_components = []
        for new_comp in plan.get('new_components', []):
            comp_name = new_comp['name']
            comp_path = component_path.parent / comp_name
            
            # Create component directory
            comp_path.mkdir(exist_ok=True)
            
            # Create CLAUDE.md for new component
            claude_md = self._generate_component_claude_md(comp_name, new_comp)
            (comp_path / "CLAUDE.md").write_text(claude_md)
            
            # Initialize git
            import subprocess
            subprocess.run(['git', 'init'], cwd=comp_path)
            
            new_components.append({
                'name': comp_name,
                'path': str(comp_path),
                'estimated_tokens': new_comp.get('estimated_tokens', 0)
            })
        
        # Move files according to plan
        # This would be done by a Claude Code sub-agent in practice
        self._migrate_files(component_path, new_components, plan)
        
        # Create contracts between new components
        self._create_contracts(new_components)
        
        # Archive original component
        archive_path = component_path.parent / f"_archived_{component_path.name}_{timestamp}"
        component_path.rename(archive_path)
        
        return {
            'action': 'split_completed',
            'original_component': component_path.name,
            'new_components': new_components,
            'backup_path': str(backup_path),
            'archive_path': str(archive_path),
            'timestamp': timestamp
        }
    
    def _generate_component_claude_md(self, name: str, component_info: Dict) -> str:
        """Generate CLAUDE.md for new split component"""
        
        return f"""# {name.replace('-', ' ').title()} Component

This component was created from splitting a larger component.

## Component Responsibility
{component_info.get('purpose', 'Component responsibilities')}

## Your Boundaries
- You work ONLY in this directory: components/{name}/
- You CANNOT access other components' source code
- You can READ contracts from ../../contracts/
- You can READ shared libraries from ../../shared-libs/

## Token Budget
- Target size: ~40,000 tokens maximum
- Current estimate: {component_info.get('estimated_tokens', 0)} tokens
- Monitor growth to avoid future splits

## Development Guidelines
1. Maintain clean, modular code
2. Write comprehensive tests
3. Document all public APIs
4. Follow established patterns
5. Alert orchestrator if approaching token limits

## Integration Points
- Read your API contract from: ../../contracts/{name}-api.yaml
- Implement all defined endpoints
- Use shared libraries for common functionality
"""
    
    def _migrate_files(self, old_path: Path, new_components: List[Dict], plan: Dict):
        """Migrate files to new components"""
        
        # This is a simplified version
        # In practice, a Claude Code sub-agent would handle the complex file migration
        
        print(f"Migrating files from {old_path.name} to new components...")
        
        # For now, just create placeholder structure
        for comp in new_components:
            comp_path = Path(comp['path'])
            (comp_path / 'src').mkdir(exist_ok=True)
            (comp_path / 'tests').mkdir(exist_ok=True)
            
            # Create README
            readme = f"""# {comp['name']}

Component split from {old_path.name}

## Setup
```bash
# Initialize development environment
```

## API
See ../../contracts/{comp['name']}-api.yaml
"""
            (comp_path / 'README.md').write_text(readme)
    
    def _create_contracts(self, components: List[Dict]):
        """Create API contracts between split components"""
        
        contracts_dir = Path("contracts")
        contracts_dir.mkdir(exist_ok=True)
        
        for comp in components:
            contract = {
                'openapi': '3.0.0',
                'info': {
                    'title': f"{comp['name']} API",
                    'version': '1.0.0',
                    'description': f"API contract for {comp['name']} component"
                },
                'paths': {}
            }
            
            contract_file = contracts_dir / f"{comp['name']}-api.yaml"
            # Would use yaml.dump in practice
            contract_file.write_text(json.dumps(contract, indent=2))
    
    def _record_split(self, original_path: Path, plan: Dict, result: Dict):
        """Record split in history"""
        
        history = []
        if self.split_history_file.exists():
            history = json.loads(self.split_history_file.read_text())
        
        history.append({
            'timestamp': datetime.now().isoformat(),
            'original_component': original_path.name,
            'split_plan': plan,
            'result': result
        })
        
        self.split_history_file.parent.mkdir(parents=True, exist_ok=True)
        self.split_history_file.write_text(json.dumps(history, indent=2))


class AutomaticSplitMonitor:
    """Monitor all components and trigger automatic splits"""
    
    def __init__(self):
        self.splitter = AdaptiveComponentSplitter()
        self.context_manager = ContextWindowManager()
    
    def monitor_all_components(self) -> List[Dict]:
        """Check all components and split as needed"""
        
        components_dir = Path("components")
        results = []
        
        for component_path in components_dir.iterdir():
            if component_path.is_dir() and not component_path.name.startswith('_'):
                # Check component
                analysis = self.context_manager.analyze_component(component_path)
                
                if analysis['status'] in ['orange', 'red']:
                    # Needs splitting
                    print(f"Auto-splitting {component_path.name}...")
                    result = self.splitter.execute_split(component_path, analysis)
                    results.append(result)
                else:
                    results.append({
                        'component': component_path.name,
                        'action': 'monitored',
                        'status': analysis['status'],
                        'tokens': analysis['estimated_tokens']
                    })
        
        return results
    
    def get_split_recommendations(self) -> List[Dict]:
        """Get recommendations for components approaching limits"""
        
        components_dir = Path("components")
        recommendations = []
        
        for component_path in components_dir.iterdir():
            if component_path.is_dir() and not component_path.name.startswith('_'):
                analysis = self.context_manager.analyze_component(component_path)
                
                if analysis['status'] == 'yellow':
                    recommendations.append({
                        'component': component_path.name,
                        'current_tokens': analysis['estimated_tokens'],
                        'percentage_used': analysis['percentage_used'],
                        'recommendation': 'Plan split strategy',
                        'urgency': 'medium'
                    })
                elif analysis['status'] in ['orange', 'red']:
                    recommendations.append({
                        'component': component_path.name,
                        'current_tokens': analysis['estimated_tokens'],
                        'percentage_used': analysis['percentage_used'],
                        'recommendation': 'Split immediately',
                        'urgency': 'high'
                    })
        
        return recommendations
```

## 4. Agent Launcher (orchestration/agent_launcher.py)

```python
import subprocess
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, Optional, List

class AgentLauncher:
    """Launch and manage Claude Code sub-agents"""
    
    def __init__(self):
        self.registry_file = Path("orchestration/agent-registry.json")
        self.max_concurrent = 3
        self._ensure_registry()
    
    def _ensure_registry(self):
        """Ensure registry exists"""
        if not self.registry_file.exists():
            self.registry_file.parent.mkdir(parents=True, exist_ok=True)
            self.registry_file.write_text(json.dumps({
                'max_concurrent': self.max_concurrent,
                'active': [],
                'queued': [],
                'completed': []
            }, indent=2))
    
    def launch_agent(self, component_name: str, task: str, 
                    priority: int = 0) -> Dict:
        """Launch a Claude Code sub-agent for a component"""
        
        registry = self._load_registry()
        
        # Check if we can launch
        if len(registry['active']) >= self.max_concurrent:
            # Queue the task
            registry['queued'].append({
                'component': component_name,
                'task': task,
                'priority': priority,
                'queued_at': datetime.now().isoformat()
            })
            self._save_registry(registry)
            
            return {
                'status': 'queued',
                'component': component_name,
                'position': len(registry['queued'])
            }
        
        # Launch the agent
        component_path = Path("components") / component_name
        
        if not component_path.exists():
            return {
                'status': 'error',
                'message': f"Component {component_name} does not exist"
            }
        
        # Prepare task file
        task_file = component_path / "current_task.md"
        task_file.write_text(f"""# Current Task

{task}

## Instructions
1. Read and understand this task
2. Implement the required functionality
3. Write comprehensive tests
4. Ensure code quality standards are met
5. Commit your work to the local git repository

## Constraints
- Work only within this component directory
- Follow the guidelines in CLAUDE.md
- Maintain token budget limits
""")
        
        # Launch Claude Code
        process = subprocess.Popen(
            ["claude", "code", "--task", f"Complete the task in current_task.md"],
            cwd=component_path,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        
        # Update registry
        registry['active'].append({
            'component': component_name,
            'task': task,
            'pid': process.pid,
            'started_at': datetime.now().isoformat()
        })
        self._save_registry(registry)
        
        return {
            'status': 'launched',
            'component': component_name,
            'pid': process.pid
        }
    
    def check_agent_status(self, component_name: str) -> Dict:
        """Check status of an agent"""
        
        registry = self._load_registry()
        
        # Check if active
        for agent in registry['active']:
            if agent['component'] == component_name:
                # Check if process is still running
                import psutil
                try:
                    process = psutil.Process(agent['pid'])
                    if process.is_running():
                        return {
                            'status': 'active',
                            'component': component_name,
                            'pid': agent['pid'],
                            'started_at': agent['started_at']
                        }
                except:
                    pass
                
                # Process ended
                self._mark_completed(component_name)
                return {'status': 'completed', 'component': component_name}
        
        # Check if queued
        for agent in registry['queued']:
            if agent['component'] == component_name:
                position = registry['queued'].index(agent) + 1
                return {
                    'status': 'queued',
                    'component': component_name,
                    'position': position
                }
        
        # Check if completed
        for agent in registry['completed']:
            if agent['component'] == component_name:
                return {
                    'status': 'completed',
                    'component': component_name,
                    'completed_at': agent.get('completed_at')
                }
        
        return {'status': 'not_found', 'component': component_name}
    
    def terminate_agent(self, component_name: str) -> Dict:
        """Terminate a running agent"""
        
        registry = self._load_registry()
        
        for agent in registry['active']:
            if agent['component'] == component_name:
                # Kill the process
                import signal
                import os
                try:
                    os.kill(agent['pid'], signal.SIGTERM)
                except:
                    pass
                
                # Update registry
                self._mark_completed(component_name)
                
                return {
                    'status': 'terminated',
                    'component': component_name
                }
        
        return {
            'status': 'not_active',
            'component': component_name
        }
    
    def process_queue(self):
        """Process queued agents if slots available"""
        
        registry = self._load_registry()
        
        # Clean up completed agents
        self._cleanup_completed()
        
        # Process queue
        while len(registry['active']) < self.max_concurrent and registry['queued']:
            # Get highest priority task
            registry['queued'].sort(key=lambda x: x['priority'], reverse=True)
            next_task = registry['queued'].pop(0)
            
            # Launch it
            self.launch_agent(
                next_task['component'],
                next_task['task'],
                next_task['priority']
            )
            
            # Reload registry for next iteration
            registry = self._load_registry()
    
    def _mark_completed(self, component_name: str):
        """Mark an agent as completed"""
        
        registry = self._load_registry()
        
        # Find and remove from active
        for i, agent in enumerate(registry['active']):
            if agent['component'] == component_name:
                agent['completed_at'] = datetime.now().isoformat()
                registry['completed'].append(agent)
                registry['active'].pop(i)
                break
        
        self._save_registry(registry)
        
        # Process queue if we have space
        self.process_queue()
    
    def _cleanup_completed(self):
        """Clean up completed agents that are no longer running"""
        
        registry = self._load_registry()
        still_active = []
        
        import psutil
        for agent in registry['active']:
            try:
                process = psutil.Process(agent['pid'])
                if process.is_running():
                    still_active.append(agent)
                else:
                    agent['completed_at'] = datetime.now().isoformat()
                    registry['completed'].append(agent)
            except:
                # Process doesn't exist
                agent['completed_at'] = datetime.now().isoformat()
                registry['completed'].append(agent)
        
        registry['active'] = still_active
        self._save_registry(registry)
    
    def _load_registry(self) -> Dict:
        """Load agent registry"""
        return json.loads(self.registry_file.read_text())
    
    def _save_registry(self, registry: Dict):
        """Save agent registry"""
        self.registry_file.write_text(json.dumps(registry, indent=2))
    
    def get_status_summary(self) -> Dict:
        """Get summary of all agents"""
        
        registry = self._load_registry()
        
        return {
            'active': len(registry['active']),
            'queued': len(registry['queued']),
            'completed': len(registry['completed']),
            'max_concurrent': self.max_concurrent,
            'active_agents': [
                {
                    'component': a['component'],
                    'task': a['task'][:50] + '...' if len(a['task']) > 50 else a['task'],
                    'started_at': a['started_at']
                }
                for a in registry['active']
            ],
            'queued_agents': [
                {
                    'component': a['component'],
                    'priority': a['priority']
                }
                for a in registry['queued']
            ]
        }
```

## 5. Migration Manager (orchestration/migration_manager.py)

```python
from pathlib import Path
from typing import Dict, List, Optional
import json
import shutil
from datetime import datetime
from claude_code_analyzer import ClaudeCodeAnalyzer

class MigrationManager:
    """Manage migration of existing projects to orchestrated architecture"""
    
    def __init__(self):
        self.analyzer = ClaudeCodeAnalyzer()
        self.migration_state_file = Path("orchestration/migration-state.json")
    
    def migrate_project(self, source_path: str, target_path: str = None) -> Dict:
        """Migrate an existing project"""
        
        source = Path(source_path)
        target = Path(target_path or "migrated_project")
        
        print(f"Starting migration of {source} to {target}")
        
        # Phase 1: Analysis
        print("Phase 1: Analyzing project structure...")
        analysis = self.analyzer.analyze_with_claude_code(
            task="migration_analysis",
            target_path=source
        )
        
        if not analysis.get('completed'):
            return {
                'status': 'failed',
                'phase': 'analysis',
                'error': analysis.get('error')
            }
        
        # Phase 2: Setup orchestration structure
        print("Phase 2: Setting up orchestration structure...")
        self._setup_orchestration_structure(target)
        
        # Phase 3: Migrate components
        print("Phase 3: Migrating components...")
        components = analysis.get('components', [])
        migrated_components = []
        
        for component in components:
            result = self._migrate_component(component, source, target)
            migrated_components.append(result)
        
        # Phase 4: Generate contracts
        print("Phase 4: Generating API contracts...")
        self._generate_contracts(migrated_components, target)
        
        # Phase 5: Setup orchestrator
        print("Phase 5: Configuring orchestrator...")
        self._setup_orchestrator(target, analysis, migrated_components)
        
        # Save migration state
        self._save_migration_state({
            'source': str(source),
            'target': str(target),
            'analysis': analysis,
            'migrated_components': migrated_components,
            'timestamp': datetime.now().isoformat()
        })
        
        return {
            'status': 'completed',
            'source': str(source),
            'target': str(target),
            'components_migrated': len(migrated_components),
            'total_lines': analysis.get('total_lines', 0)
        }
    
    def _setup_orchestration_structure(self, target: Path):
        """Create orchestration directory structure"""
        
        # Create directories
        (target / "components").mkdir(parents=True, exist_ok=True)
        (target / "contracts").mkdir(parents=True, exist_ok=True)
        (target / "shared-libs").mkdir(parents=True, exist_ok=True)
        (target / "orchestration").mkdir(parents=True, exist_ok=True)
        
        # Copy orchestration tools
        # In practice, these would be copied from the orchestration template
        tools = [
            "context_manager.py",
            "agent_launcher.py",
            "component_splitter.py",
            "claude_code_analyzer.py"
        ]
        
        for tool in tools:
            tool_content = f"""# {tool}
# Orchestration tool
# Full implementation would be copied here
"""
            (target / "orchestration" / tool).write_text(tool_content)
    
    def _migrate_component(self, component: Dict, source: Path, target: Path) -> Dict:
        """Migrate a single component"""
        
        comp_name = component['name']
        comp_source = Path(component.get('path', source / comp_name))
        comp_target = target / "components" / comp_name
        
        # Create component directory
        comp_target.mkdir(parents=True, exist_ok=True)
        
        # Copy component files
        if comp_source.exists() and comp_source.is_dir():
            # Copy entire directory
            shutil.copytree(comp_source, comp_target, dirs_exist_ok=True)
        else:
            # Create from file list if provided
            (comp_target / "src").mkdir(exist_ok=True)
            
        # Create component CLAUDE.md
        claude_md = self._generate_migrated_component_claude_md(component)
        (comp_target / "CLAUDE.md").write_text(claude_md)
        
        # Initialize git
        import subprocess
        subprocess.run(['git', 'init'], cwd=comp_target, capture_output=True)
        subprocess.run(['git', 'add', '.'], cwd=comp_target, capture_output=True)
        subprocess.run(
            ['git', 'commit', '-m', 'Initial migration'],
            cwd=comp_target,
            capture_output=True
        )
        
        return {
            'name': comp_name,
            'source': str(comp_source),
            'target': str(comp_target),
            'lines': component.get('lines', 0),
            'type': component.get('type', 'unknown')
        }
    
    def _generate_migrated_component_claude_md(self, component: Dict) -> str:
        """Generate CLAUDE.md for migrated component"""
        
        return f"""# {component['name'].replace('-', ' ').title()} Component

This component was migrated from an existing project.

## Component Type
{component.get('type', 'general')}

## Migration Status
- Source lines: {component.get('lines', 'unknown')}
- Estimated tokens: {component.get('estimated_tokens', 'unknown')}
- Migration date: {datetime.now().isoformat()}

## Your Boundaries
- You work ONLY in this directory
- You CANNOT access other components' source code
- You can READ contracts from ../../contracts/
- You can READ shared libraries from ../../shared-libs/

## Priority Tasks
1. Review migrated code for issues
2. Add missing tests (target 80% coverage)
3. Improve documentation
4. Refactor for better maintainability
5. Ensure API contract compliance

## Development Standards
- Follow existing code style
- Maintain backward compatibility during migration
- Document all changes
- Commit regularly to local git
"""
    
    def _generate_contracts(self, components: List[Dict], target: Path):
        """Generate API contracts for migrated components"""
        
        contracts_dir = target / "contracts"
        
        for comp in components:
            # Use Claude Code to extract API definitions
            api_analysis = self.analyzer.analyze_with_claude_code(
                task="extract_api",
                target_path=Path(comp['target'])
            )
            
            # Create contract file
            contract = {
                'openapi': '3.0.0',
                'info': {
                    'title': f"{comp['name']} API",
                    'version': '1.0.0',
                    'description': f"Migrated API for {comp['name']}"
                },
                'paths': api_analysis.get('paths', {})
            }
            
            contract_file = contracts_dir / f"{comp['name']}-api.yaml"
            contract_file.write_text(json.dumps(contract, indent=2))
    
    def _setup_orchestrator(self, target: Path, analysis: Dict, 
                          components: List[Dict]):
        """Set up master orchestrator"""
        
        orchestrator_md = f"""# Master Orchestrator - Migrated Project

## Project Migration Summary
- Original type: {analysis.get('project_type', 'unknown')}
- Total lines migrated: {analysis.get('total_lines', 0):,}
- Components created: {len(components)}
- Migration date: {datetime.now().isoformat()}

## Components
"""
        
        for comp in components:
            orchestrator_md += f"- **{comp['name']}** ({comp['type']}): {comp['lines']:,} lines\n"
        
        orchestrator_md += """

## Your Role
You are the master orchestrator for this migrated project. You coordinate all work but NEVER write production code directly.

## Operating Principles
1. All new development through sub-agents
2. Maximum 3 concurrent sub-agents
3. Strict component isolation
4. Contract-based communication
5. Automatic splitting at 70,000 tokens

## Migration Completion Tasks
1. Validate all components are functional
2. Run integration tests across components
3. Identify and fix migration issues
4. Optimize component boundaries
5. Document system architecture

## Monitoring
- Check component sizes: `python orchestration/context_manager.py`
- View agent status: `python orchestration/agent_launcher.py status`
- Split recommendations: `python orchestration/component_splitter.py recommend`
"""
        
        (target / "CLAUDE.md").write_text(orchestrator_md)
    
    def _save_migration_state(self, state: Dict):
        """Save migration state for recovery"""
        
        self.migration_state_file.parent.mkdir(parents=True, exist_ok=True)
        self.migration_state_file.write_text(json.dumps(state, indent=2))
```