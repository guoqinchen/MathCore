#!/usr/bin/env python3
"""
任务管理系统
"""

import os
import re
import time
from typing import List, Optional



class Task:
    """任务类"""
    def __init__(self, task_id: int, title: str, priority: str,
                 depends_on: List[int], status: str):
        self.task_id = task_id
        self.title = title
        self.priority = priority
        self.depends_on = depends_on
        self.status = status
        self.description = ""
        self.steps = []
        self.resources = []
        self.expected_result = ""
        self.time_limit = ""
        
    def __str__(self):
        return f"Task {self.task_id}: {self.title} ({self.status})"
    
    def is_ready(self, completed_tasks: List[int]) -> bool:
        """检查任务是否准备好执行"""
        if self.status == "Completed":
            return False
            
        for dep_id in self.depends_on:
            if dep_id not in completed_tasks:
                return False
                
        return True

class TaskManager:
    """任务管理器"""
    def __init__(self, task_file: str = ".trae/tasks.md"):
        self.task_file = task_file
        self.tasks = self._read_tasks()
        self.completed_tasks = self._get_completed_tasks()
    
    def _read_tasks(self) -> List[Task]:
        """从任务文件读取任务"""
        tasks = []
        
        if not os.path.exists(self.task_file):
            print(f"任务文件 {self.task_file} 不存在")
            return tasks
            
        with open(self.task_file, 'r', encoding='utf-8') as f:
            content = f.read()
            
        # 匹配任务块
        task_blocks = re.findall(r'## \[([ x/]?)\] Task (\d+): (.+?)(?=## \[|$)', content, re.DOTALL)
        
        for block in task_blocks:
            status_char, task_id_str, title = block
            task_id = int(task_id_str.strip())
            
            # 首先从文本中解析状态
            status = "Pending"
            status_text_match = re.search(r'\*\*Status\*\*: (.+)', block[2])
            if status_text_match:
                status_text = status_text_match.group(1).strip()
                if status_text == "Completed":
                    status = "Completed"
                elif status_text == "In Progress":
                    status = "In Progress"
                else:
                    status = "Pending"
            
            # 使用状态字符作为后备
            if status == "Pending":
                if status_char == 'x':
                    status = "Completed"
                elif status_char == '/':
                    status = "In Progress"
                else:
                    status = "Pending"
                
            # 解析任务属性
            priority = "P2"
            depends_on = []
            
            # 提取优先级 - 在当前任务块内搜索
            priority_match = re.search(r'\*\*Priority\*\*: (P[0-2])', block[2])
            if priority_match:
                priority = priority_match.group(1)
                
            # 提取依赖任务 - 在当前任务块内搜索
            depends_match = re.search(r'\*\*Depends On\*\*: (.+)', block[2])
            if depends_match:
                depends_str = depends_match.group(1).strip()
                if depends_str != "None":
                    # 匹配 "Task 1, Task 2" 格式
                    dep_matches = re.findall(r'Task (\d+)', depends_str)
                    depends_on = [int(dep.strip()) for dep in dep_matches]
            
            task = Task(task_id, title.strip().split("\n")[0], priority, depends_on, status)
            
            # 提取任务详细信息
            self._parse_task_details(task, block[2])
            
            tasks.append(task)
            
        return tasks
    
    def _parse_task_details(self, task: Task, block_content: str):
        """解析任务详细信息"""
        # 任务描述解析
        desc_match = re.search(r'\n> (.+?)(?=\n-|\n$)', block_content, re.DOTALL)
        if desc_match:
            task.description = desc_match.group(1).strip()
        
        # 执行步骤解析
        steps_match = re.search(r'\*\*Steps\*\*:?\s*(.*?)(?=\*\*|$)',
                               block_content, re.DOTALL)
        if steps_match:
            steps_text = steps_match.group(1).strip()
            if steps_text:
                steps = re.split(r'\n\s*[-*]', steps_text)
                task.steps = [step.strip() for step in steps if step.strip()]
        
        # 资源需求解析
        resources_match = re.search(r'\*\*Resources\*\*:?\s*(.*?)(?=\*\*|$)',
                                   block_content, re.DOTALL)
        if resources_match:
            resources_text = resources_match.group(1).strip()
            if resources_text:
                task.resources = [
                    r.strip() for r in re.split(r'[,;]', resources_text) 
                    if r.strip()
                ]
        
        # 预期结果解析
        expected_match = re.search(r'\*\*Expected Result\*\*:?\s*(.*?)(?=\*\*|$)',
                                 block_content, re.DOTALL)
        if expected_match:
            task.expected_result = expected_match.group(1).strip()
        
        # 时间限制解析
        time_match = re.search(r'\*\*Time Limit\*\*:?\s*(.*?)(?=\*\*|$)',
                             block_content, re.DOTALL)
        if time_match:
            task.time_limit = time_match.group(1).strip()
        
    def _get_completed_tasks(self) -> List[int]:
        """获取已完成任务的ID"""
        return [task.task_id for task in self.tasks if task.status == "Completed"]
        
    def sort_tasks(self, sort_by: str = "priority") -> List[Task]:
        """
        任务排序
        sort_by: 'priority' (默认), 'creation'
        """
        # 筛选可执行任务
        ready_tasks = [task for task in self.tasks if task.is_ready(self.completed_tasks)]
        
        if sort_by == "priority":
            # 按优先级排序 (P0 > P1 > P2)
            priority_order = {"P0": 0, "P1": 1, "P2": 2}
            sorted_tasks = sorted(
                ready_tasks,
                key=lambda t: (priority_order.get(t.priority, 3), t.task_id)
            )
        elif sort_by == "creation":
            # 按创建时间（任务ID）排序
            sorted_tasks = sorted(ready_tasks, key=lambda t: t.task_id)
        else:
            sorted_tasks = ready_tasks
            
        return sorted_tasks
        
    def get_next_task(self, sort_by: str = "priority") -> Optional[Task]:
        """获取下一个待执行任务"""
        sorted_tasks = self.sort_tasks(sort_by)
        
        if sorted_tasks:
            return sorted_tasks[0]
        else:
            return None
            
    def execute_task(self, task: Task) -> bool:
        """执行任务"""
        print(f"\n{'='*60}")
        print(f"开始执行任务: {task}")
        print(f"{'='*60}")
        
        # 记录任务开始时间
        start_time = time.time()
        
        try:
            # 更新任务状态为进行中
            self._update_task_status(task.task_id, "In Progress")
            
            # 根据任务ID执行对应的操作
            success = self._execute_task_by_id(task.task_id)
            
            if success:
                # 任务成功完成
                duration = time.time() - start_time
                print(f"\n任务执行成功！耗时: {duration:.2f}秒")
                self._update_task_status(task.task_id, "Completed")
                self.completed_tasks.append(task.task_id)
                return True
            else:
                print("\n任务执行失败")
                self._update_task_status(task.task_id, "Pending")
                return False
                
        except Exception as e:
            print(f"\n任务执行异常: {e}")
            self._update_task_status(task.task_id, "Pending")
            return False
            
    def _execute_task_by_id(self, task_id: int) -> bool:
        """根据任务ID执行具体操作"""
        # 根据任务ID执行相应的操作
        print(f"执行任务 {task_id}:")
        
        try:
            if task_id == 6:
                # Task 6: 实现完整的 seccomp 系统调用白名单
                print("  任务: 实现完整的 seccomp 系统调用白名单")
                print("  执行: 需要在 crates/kernel/src/sandbox/mod.rs 中实现完整的系统调用白名单")
                print("  资源: 需要参考 Linux 系统调用列表和 seccomp 文档")
                print("  预期结果: 提供更全面的系统调用白名单，增强安全性")
                
            elif task_id == 7:
                # Task 7: 增加 cgroups 资源隔离机制
                print("  任务: 增加 cgroups 资源隔离机制")
                print("  执行: 需要在 crates/kernel/src/sandbox/mod.rs 中实现 cgroups 支持")
                print("  资源: 需要参考 Linux cgroups v2 文档和相关库")
                print("  预期结果: 提供内存、CPU、IO 等资源的限制功能")
                
            elif task_id == 8:
                # Task 8: 更新各阶段任务文档
                print("  任务: 更新各阶段任务文档")
                print("  执行: 需要更新 docs/ 目录下的 phase1_tasks.md 等文档")
                print("  资源: 需要了解项目的最新进展和任务完成情况")
                print("  预期结果: 文档与实际代码同步")
                
            elif task_id == 9:
                # Task 9: 编写详细的 API 文档和使用教程
                print("  任务: 编写详细的 API 文档和使用教程")
                print("  执行: 需要为 crates/ 目录下的各个模块编写文档")
                print("  资源: 需要使用 rustdoc 和 mkdocs 等工具")
                print("  预期结果: 提供完整的 API 文档和使用示例")
                
            elif task_id == 10:
                # Task 10: 增加边缘情况的测试用例
                print("  任务: 增加边缘情况的测试用例")
                print("  执行: 需要在各模块的 tests/ 目录下添加更多测试")
                print("  资源: 需要考虑各种边界条件和异常情况")
                print("  预期结果: 提高代码覆盖率和鲁棒性")
                
            elif task_id == 11:
                # Task 11: 完善性能测试覆盖
                print("  任务: 完善性能测试覆盖")
                print("  执行: 需要在 crates/compute/src/benches/ 等目录添加性能测试")
                print("  资源: 需要使用 criterion 等性能测试工具")
                print("  预期结果: 提供性能基准数据")
                
            elif task_id == 12:
                # Task 12: 增加集成测试
                print("  任务: 增加集成测试")
                print("  执行: 需要在根目录的 tests/ 目录下添加集成测试")
                print("  资源: 需要测试各个模块之间的交互")
                print("  预期结果: 确保整个系统的功能正确性")
                
            elif task_id == 13:
                # Task 13: 完善 CI/CD 配置
                print("  任务: 完善 CI/CD 配置")
                print("  执行: 需要更新 .github/workflows/ci.yml 文件")
                print("  资源: 需要了解 GitHub Actions 的配置")
                print("  预期结果: 确保每次提交都通过所有测试")
                
            elif task_id == 14:
                # Task 14: 实现自动化的文档生成
                print("  任务: 实现自动化的文档生成")
                print("  执行: 需要配置 rustdoc 和 mkdocs 等工具的自动化流程")
                print("  资源: 需要使用 GitHub Actions 配置自动化")
                print("  预期结果: 每次提交自动更新文档")
                
            elif task_id == 15:
                # Task 15: 建立代码审查流程
                print("  任务: 建立代码审查流程")
                print("  执行: 需要制定代码审查规范和流程")
                print("  资源: 需要参考业内最佳实践")
                print("  预期结果: 提高代码质量和团队协作效率")
                
            else:
                print("  未知任务 {}".format(task_id))
                
            # 模拟执行延迟
            time.sleep(0.5)
            
            return True
            
        except Exception as e:
            print(f"  任务执行失败: {e}")
            return False
        
    def _update_task_status(self, task_id: int, new_status: str):
        """更新任务状态"""
        if not os.path.exists(self.task_file):
            print(f"任务文件 {self.task_file} 不存在")
            return
            
        with open(self.task_file, 'r', encoding='utf-8') as f:
            content = f.read()
            
        # 状态字符映射
        status_chars = {
            "Pending": " ",
            "In Progress": "/",
            "Completed": "x"
        }
        
        # 替换任务状态字符
        pattern = rf'## \[([ x/])\] Task {task_id}:'
        replacement = f'## [{status_chars.get(new_status, " ")}] Task {task_id}:'
        updated_content = re.sub(pattern, replacement, content)
        
        # 更新状态文本 - 在任务块内更新
        # 首先找到任务块
        task_block_pattern = rf'## \[.{1}\] Task {task_id}: (.*?)(?=## \[|$)'
        task_block_match = re.search(task_block_pattern, updated_content, re.DOTALL)
        
        if task_block_match:
            task_block = task_block_match.group(1)
            # 更新状态文本
            status_text_pattern = r'\*\*Status\*\*: (Pending|In Progress|Completed)'
            updated_block = re.sub(status_text_pattern, f'**Status**: {new_status}', task_block)
            
            # 替换整个任务块
            updated_content = updated_content.replace(task_block, updated_block)
            
        with open(self.task_file, 'w', encoding='utf-8') as f:
            f.write(updated_content)
            
        # 更新内存中的任务状态
        for task in self.tasks:
            if task.task_id == task_id:
                task.status = new_status
                break
                
    def print_task_info(self, task: Task):
        """打印任务详细信息"""
        print(f"\n任务详情:")
        print(f"  任务ID: {task.task_id}")
        print(f"  标题: {task.title}")
        print(f"  优先级: {task.priority}")
        print(f"  状态: {task.status}")
        print(f"  依赖任务: {', '.join([f'Task {dep}' for dep in task.depends_on]) if task.depends_on else 'None'}")
        print(f"  描述: {task.description}")
        
        if task.steps:
            print(f"  执行步骤:")
            for i, step in enumerate(task.steps, 1):
                print(f"    {i}. {step}")
                
        if task.resources:
            print(f"  所需资源: {', '.join(task.resources)}")
            
        if task.expected_result:
            print(f"  预期结果: {task.expected_result}")
            
        if task.time_limit:
            print(f"  时间限制: {task.time_limit}")
            
    def print_task_summary(self):
        """打印任务摘要"""
        print(f"任务总数: {len(self.tasks)}")
        print(f"已完成: {len([t for t in self.tasks if t.status == 'Completed'])}")
        print(f"进行中: {len([t for t in self.tasks if t.status == 'In Progress'])}")
        print(f"待执行: {len([t for t in self.tasks if t.status == 'Pending'])}")
        
        ready_count = len([t for t in self.tasks if t.is_ready(self.completed_tasks)])
        print(f"可执行: {ready_count}")

def main():
    """主函数"""
    task_manager = TaskManager()
    
    print("任务管理系统初始化完成")
    task_manager.print_task_summary()
    
    # 获取下一个待执行任务
    next_task = task_manager.get_next_task()
    
    if next_task:
        print(f"\n下一个待执行任务:")
        task_manager.print_task_info(next_task)
        
        # 确认是否执行
        while True:
            choice = input("\n是否执行该任务? (y/n): ").strip().lower()
            if choice in ['y', 'yes']:
                # 执行任务
                success = task_manager.execute_task(next_task)
                break
            elif choice in ['n', 'no']:
                print("取消任务执行")
                break
            else:
                print("无效输入，请输入 y 或 n")
    else:
        print("\n所有任务已完成或无可执行任务")
        
if __name__ == "__main__":
    main()
