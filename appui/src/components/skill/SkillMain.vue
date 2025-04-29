<script setup>
import { callSkillList, callSkillDelete } from '@/call';
import AddSkillDialog from '@/components/skill/AddSkillDialog.vue';
import SkillCard from '@/components/skill/SkillCard.vue';
import SkillsetCard from '@/components/skill/SkillsetCard.vue';
import ViewSkillDialog from '@/components/skill/ViewSkillDialog.vue';
import { useAiStore } from '@/stores/ai';
import { useToolStore } from '@/stores/tool';
import { computed, onMounted, reactive, ref, watch } from 'vue';

const visible = defineModel('visible');
const aiStore = useAiStore();
const toolStore = useToolStore();

const refScroll = ref();
const state = reactive({
  search: '',
  group: false,
  skills: [],
  deletingSkillIds: [],
  deletingSkillsetIds: [],

  requestingFetchSkills: false,
  hasMoreSkills: false,

  showAddSkillDialog: false,
  requestingAddSkill: false,

  skillSelected: null,
  showViewSkillDialog: false,
});

const skillsets = computed(() => {
  const skillsets = [];

  for (const skill of state.skills) {
    const tool = toolStore.getById(skill.tool_id);
    if (tool) {
      let skillset = skillsets.find((d) => d.toolset_id === tool.toolset_id);
      if (skillset) {
        skillset.skills.push(skill);
      } else {
        skillsets.push({ toolset_id: tool.toolset_id, toolset_title: tool.toolset_title, skills: [skill] });
      }
    }
  }

  return skillsets;
});

const pageSize = 20;

const fetchSkills = async () => {
  state.requestingFetchSkills = true;

  try {
    let skills = await callSkillList(aiStore.getActiveName(), state.search, pageSize + 1, state.skills.length);
    if (skills.length > pageSize) {
      state.skills = state.skills.concat(skills.splice(0, pageSize));
      state.hasMoreSkills = true;
    } else {
      state.skills = state.skills.concat(skills);
      state.hasMoreSkills = false;
    }

    let toolIds = skills.map((skill) => skill.tool_id);
    if (toolIds.length > 0) {
      await toolStore.fetchToolsByIds(toolIds);
    }
  } finally {
    state.requestingFetchSkills = false;
  }
};

const refreshSkills = async () => {
  state.skills = [];
  state.hasMoreSkills = false;

  await fetchSkills();
};

const refreshSkillsReset = async () => {
  state.search = '';
  await refreshSkills();
};

const onDeleteSkill = async (skill) => {
  state.deletingSkillIds.push(skill.id);

  try {
    await callSkillDelete(aiStore.getActiveName(), skill.id);
    state.skills = state.skills.filter((d) => d.id !== skill.id);
  } finally {
    state.deletingSkillIds = state.deletingSkillIds.filter((id) => id !== skill.id);
  }
};

const onDeleteSkillset = async (skillset) => {
  state.deletingSkillsetIds.push(skillset.toolset_id);

  try {
    for (const skill of skillset.skills) {
      await onDeleteSkill(skill);
    }
  } finally {
    state.deletingSkillsetIds = state.deletingSkillsetIds.filter((id) => id !== skillset.toolset_id);
  }
};

const onScroll = async () => {
  const wrap = refScroll.value;
  if (wrap) {
    if (
      wrap.scrollTop + wrap.clientHeight >= wrap.scrollHeight - 2 &&
      !state.requestingFetchSkills &&
      state.hasMoreSkills
    ) {
      await fetchSkills();
    }
  }
};

const onViewSkill = (skill) => {
  state.skillSelected = skill;
  state.showViewSkillDialog = true;
};

watch(
  () => aiStore.getActiveName(),
  async () => {
    state.search = '';

    await refreshSkills();
  },
);

onMounted(async () => {
  await refreshSkills();
});
</script>

<template>
  <div class="flex h-full w-full" v-show="visible">
    <div class="flex h-full flex-1 flex-col items-center gap-2 px-2">
      <div class="my-4 flex w-full gap-2 px-2">
        <div class="flex gap-2">
          <el-input class="lg:w-100 w-40 sm:w-60 md:w-80" v-model="state.search" @keyup.enter.prevent="refreshSkills">
            <template #prefix>
              <template v-if="state.requestingFetchSkills">
                <el-icon class="rotating"><i class="ri-loader-2-line"></i></el-icon>
              </template>
              <template v-else>
                <el-icon><i class="ri-search-line"></i></el-icon>
              </template>
            </template>

            <template v-if="!!state.search.trim()" #append>
              <el-button @click="refreshSkillsReset">
                <el-icon><i class="ri-close-line"></i></el-icon>
              </el-button>
            </template>
          </el-input>

          <el-button :disabled="state.requestingFetchSkills" @click="refreshSkills">
            <template v-if="state.requestingFetchSkills">
              <el-icon class="rotating"><i class="ri-loader-2-line"></i></el-icon>
            </template>
            <template v-else>
              <el-icon><i class="ri-refresh-line"></i></el-icon>
            </template>
          </el-button>
        </div>

        <span class="flex-1"></span>

        <el-button :disabled="state.requestingAddSkill" @click="state.showAddSkillDialog = true" type="primary">
          <template v-if="state.requestingAddSkill">
            <el-icon class="rotating"><i class="ri-loader-4-line"></i></el-icon>
          </template>
          <template v-else>
            <el-icon><i class="ri-add-line"></i></el-icon>
          </template>
        </el-button>
      </div>

      <div class="flex w-full flex-1 flex-col items-center overflow-auto px-2" ref="refScroll" @scroll="onScroll">
        <template v-if="state.skills.length > 0">
          <div
            class="grid w-full grid-flow-row grid-cols-1 gap-4 pb-8 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5"
          >
            <template v-if="state.group">
              <SkillsetCard
                v-for="skillset in skillsets"
                :deleting="state.deletingSkillsetIds.includes(skillset.toolset_id)"
                :skillset="skillset"
                :key="skillset.toolset_id"
                @ungroup="state.group = false"
                @delete="onDeleteSkillset(skillset)"
              />
            </template>
            <template v-else>
              <SkillCard
                v-for="skill in state.skills"
                :deleting="state.deletingSkillIds.includes(skill.id)"
                :skill="skill"
                :key="skill.id"
                @delete="onDeleteSkill(skill)"
                @group="state.group = true"
                @view="onViewSkill(skill)"
              />
            </template>
          </div>
        </template>
        <template v-else>
          <div v-if="!state.requestingFetchSkills">
            <el-empty :description="$t('label.empty')" />
          </div>
        </template>
      </div>
    </div>
  </div>

  <AddSkillDialog v-model:visible="state.showAddSkillDialog" @added="refreshSkillsReset" />
  <ViewSkillDialog
    v-model:visible="state.showViewSkillDialog"
    :skill="state.skillSelected"
    @delete="onDeleteSkill(state.skillSelected)"
  />
</template>
